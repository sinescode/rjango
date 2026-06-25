/// Django-like paginator for splitting query results into pages.
#[derive(Debug, Clone)]
pub struct Paginator {
    count: usize,
    per_page: usize,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub number: usize,
    pub has_previous: bool,
    pub has_next: bool,
    pub num_pages: usize,
    pub start_index: usize,
    pub end_index: usize,
}

impl Paginator {
    pub fn new(count: usize, per_page: usize) -> Self {
        Self { count, per_page }
    }

    pub fn num_pages(&self) -> usize {
        if self.count == 0 { return 0; }
        (self.count + self.per_page - 1) / self.per_page
    }

    pub fn page_range(&self) -> Vec<usize> {
        (1..=self.num_pages()).collect()
    }

    pub fn page(&self, number: usize) -> std::result::Result<Page, String> {
        if number == 0 || number > self.num_pages() {
            return Err(format!("Page {} out of range (1-{})", number, self.num_pages()));
        }
        let start_index = (number - 1) * self.per_page;
        let end_index = std::cmp::min(start_index + self.per_page, self.count);
        Ok(Page {
            number,
            has_previous: number > 1,
            has_next: number < self.num_pages(),
            num_pages: self.num_pages(),
            start_index,
            end_index,
        })
    }

    pub fn count(&self) -> usize { self.count }
    pub fn per_page(&self) -> usize { self.per_page }
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
    }
}
