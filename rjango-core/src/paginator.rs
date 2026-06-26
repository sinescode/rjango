/// Django-like paginator for splitting query results into pages.
/// Mirrors `django.core.paginator`.

use std::sync::Arc;

/// Error returned when a page number is out of range.
#[derive(Debug, Clone)]
pub struct InvalidPage;

impl std::fmt::Display for InvalidPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid page")
    }
}

impl std::error::Error for InvalidPage {}

/// Error returned when the page number is not an integer.
#[derive(Debug, Clone)]
pub struct PageNotAnInteger;

impl std::fmt::Display for PageNotAnInteger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Page not an integer")
    }
}

impl std::error::Error for PageNotAnInteger {}

/// Error returned when the page is empty.
#[derive(Debug, Clone)]
pub struct EmptyPage;

impl std::fmt::Display for EmptyPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Empty page")
    }
}

impl std::error::Error for EmptyPage {}

#[derive(Debug, Clone)]
pub struct Paginator {
    count: usize,
    per_page: usize,
    orphans: usize,
    allow_empty_first_page: bool,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub number: usize,
    pub has_previous: bool,
    pub has_next: bool,
    pub num_pages: usize,
    pub start_index: usize,
    pub end_index: usize,
    pub object_list: Vec<String>, // placeholder for actual objects
    pub paginator: Arc<Paginator>,
}

impl Paginator {
    pub fn new(count: usize, per_page: usize) -> Self {
        Self { count, per_page, orphans: 0, allow_empty_first_page: true }
    }

    /// Set the orphans value — minimum number of items on the last page.
    /// If the last page would have ≤ orphans items, merge them into the previous page.
    pub fn orphans(mut self, n: usize) -> Self {
        self.orphans = n;
        self
    }

    /// Set whether the first page can be empty (when count is 0).
    pub fn allow_empty_first_page(mut self, val: bool) -> Self {
        self.allow_empty_first_page = val;
        self
    }

    pub fn num_pages(&self) -> usize {
        if self.count == 0 { return 0; }

        let mut pages = (self.count + self.per_page - 1) / self.per_page;
        if pages == 0 { return 1; }

        // Check if the last page would have fewer items than orphans
        if self.orphans > 0 && pages > 1 {
            let items_on_last_page = self.count - (pages - 1) * self.per_page;
            if items_on_last_page <= self.orphans {
                pages -= 1;
            }
        }
        pages
    }

    pub fn page_range(&self) -> Vec<usize> {
        let n = self.num_pages();
        if n == 0 { return vec![]; }
        (1..=n).collect()
    }

    pub fn page(&self, number: usize) -> std::result::Result<Page, String> {
        let n = self.num_pages();
        if number == 0 || (number > n && n > 0) {
            return Err(format!("Page {} out of range (1-{})", number, n));
        }
        // Special case: if count is 0 and first page is allowed
        if self.count == 0 && self.allow_empty_first_page && number == 1 {
            let paginator = Arc::new(self.clone());
            return Ok(Page {
                number: 1,
                has_previous: false,
                has_next: false,
                num_pages: 1,
                start_index: 0,
                end_index: 0,
                object_list: vec![],
                paginator,
            });
        }

        let start_index = (number - 1) * self.per_page;
        let mut end_index = std::cmp::min(start_index + self.per_page, self.count);
        // If orphans caused last pages to merge, the now-last page gets all remaining items
        if self.orphans > 0 && n > 1 && number == n {
            let raw_leftover = self.count - n * self.per_page;
            if raw_leftover > 0 && raw_leftover <= self.orphans {
                end_index = self.count;
            }
        }
        let paginator = Arc::new(self.clone());
        Ok(Page {
            number,
            has_previous: number > 1,
            has_next: number < n,
            num_pages: n,
            start_index,
            end_index,
            object_list: vec![],
            paginator,
        })
    }

    pub fn count(&self) -> usize { self.count }
    pub fn per_page(&self) -> usize { self.per_page }

    /// Get the number of items on a specific page.
    pub fn page_items_count(&self, number: usize) -> Option<usize> {
        let page = self.page(number).ok()?;
        Some(page.end_index - page.start_index)
    }

    /// Get a page (alias for `page()`).
    pub fn get_page(&self, number: u32) -> std::result::Result<Page, String> {
        self.page(number as usize)
    }

    /// Validate and normalize a page number.
    pub fn validate_number(&self, number: u32) -> u32 {
        let n = self.num_pages() as u32;
        if number == 0 || number > n {
            1
        } else {
            number
        }
    }
}

impl Page {
    /// Returns true if there is a next page.
    pub fn has_next(&self) -> bool {
        self.has_next
    }

    /// Returns true if there is a previous page.
    pub fn has_previous(&self) -> bool {
        self.has_previous
    }

    /// Returns true if there are other pages (previous or next).
    pub fn has_other_pages(&self) -> bool {
        self.has_previous || self.has_next
    }

    /// Returns the next page number, or None if there is no next.
    pub fn next_page_number(&self) -> Option<u32> {
        if self.has_next {
            Some(self.number as u32 + 1)
        } else {
            None
        }
    }

    /// Returns the previous page number, or None if there is no previous.
    pub fn previous_page_number(&self) -> Option<u32> {
        if self.has_previous {
            Some(self.number as u32 - 1)
        } else {
            None
        }
    }

    /// Returns the 1-based start index for items on this page.
    pub fn start_index(&self) -> u32 {
        self.start_index as u32
    }

    /// Returns the 1-based end index (exclusive) for items on this page.
    pub fn end_index(&self) -> u32 {
        self.end_index as u32
    }

    /// Returns a reference to the object list.
    pub fn object_list(&self) -> &[String] {
        &self.object_list
    }

    /// Returns a reference to the owning Paginator.
    pub fn paginator_ref(&self) -> &Paginator {
        &self.paginator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginator_basic() {
        let p = Paginator::new(100, 10);
        assert_eq!(p.num_pages(), 10);
        let page = p.page(1).unwrap();
        assert!(!page.has_previous);
        assert!(page.has_next);
        assert_eq!(page.start_index, 0);
        assert_eq!(page.end_index, 10);
    }

    #[test]
    fn test_paginator_partial() {
        let p = Paginator::new(95, 10);
        assert_eq!(p.num_pages(), 10);
        let last = p.page(10).unwrap();
        assert_eq!(last.start_index, 90);
        assert_eq!(last.end_index, 95);
    }

    #[test]
    fn test_paginator_invalid() {
        let p = Paginator::new(10, 5);
        assert!(p.page(0).is_err());
        assert!(p.page(3).is_err());
    }

    #[test]
    fn test_empty() {
        let p = Paginator::new(0, 10);
        assert_eq!(p.num_pages(), 0);
        assert!(p.page(1).is_ok()); // allow_empty_first_page = true
    }

    #[test]
    fn test_empty_disallow() {
        let p = Paginator::new(0, 10).allow_empty_first_page(false);
        assert_eq!(p.num_pages(), 0);
        // When allow_empty_first_page is false, page 1 is invalid for empty
        assert!(p.page_range().is_empty());
    }

    #[test]
    fn test_num_pages_single() {
        let p = Paginator::new(1, 10);
        assert_eq!(p.num_pages(), 1);
        let page = p.page(1).unwrap();
        assert!(!page.has_next);
        assert!(!page.has_previous);
    }

    #[test]
    fn test_orphans_merge_last_page() {
        let p = Paginator::new(21, 10).orphans(2);
        // Without orphans: 3 pages (10, 10, 1). With orphans(2): last page has 1 <= 2 → merged
        assert_eq!(p.num_pages(), 2);
        // Page 1: 10 items, Page 2: 11 items (1+3 merged)
        assert_eq!(p.page_items_count(2), Some(11));
    }

    #[test]
    fn test_orphans_no_merge() {
        let p = Paginator::new(23, 10).orphans(2);
        // Last page has 3 items, which is > orphans(2), so keep 3 pages
        assert_eq!(p.num_pages(), 3);
    }

    #[test]
    fn test_page_range() {
        let p = Paginator::new(50, 10);
        assert_eq!(p.page_range(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_page_range_empty() {
        let p = Paginator::new(0, 10);
        assert!(p.page_range().is_empty());
    }

    #[test]
    fn test_page_end_index() {
        let p = Paginator::new(33, 10);
        let last = p.page(4).unwrap();
        assert_eq!(last.start_index, 30);
        assert_eq!(last.end_index, 33);
    }

    #[test]
    fn test_count_and_per_page() {
        let p = Paginator::new(42, 7);
        assert_eq!(p.count(), 42);
        assert_eq!(p.per_page(), 7);
    }
}
