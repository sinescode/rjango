/// Functional programming utilities.
/// Lazy evaluation, memoization, etc. (like django.utils.functional)

/// Simple memoization decorator.
pub fn memoize<A, B, F>(f: F) -> impl Fn(A) -> B
where
    A: std::hash::Hash + Eq + Clone,
    B: Clone,
    F: Fn(A) -> B,
{
    use std::cell::RefCell;
    use std::collections::HashMap;

    let cache: RefCell<HashMap<A, B>> = RefCell::new(HashMap::new());
    move |arg| {
        let mut cache = cache.borrow_mut();
        if let Some(result) = cache.get(&arg) {
            return result.clone();
        }
        let result = f(arg.clone());
        cache.insert(arg, result.clone());
        result
    }
}

/// Partition a slice into two vectors based on a predicate.
pub fn partition<T, F>(items: &[T], f: F) -> (Vec<&T>, Vec<&T>)
where F: Fn(&T) -> bool {
    let mut trues = Vec::new();
    let mut falses = Vec::new();
    for item in items {
        if f(item) { trues.push(item); } else { falses.push(item); }
    }
    (trues, falses)
}

/// Thread-safe cached property — computes once, caches result.
/// Like Django's `cached_property` decorator.
/// Usage: `CachedProperty::new(|| expensive_computation())`
pub struct CachedProperty<T> {
    value: std::cell::OnceCell<T>,
    compute: fn() -> T,
}

impl<T> CachedProperty<T> {
    pub const fn new(compute: fn() -> T) -> Self {
        Self {
            value: std::cell::OnceCell::new(),
            compute,
        }
    }

    pub fn get(&self) -> &T {
        self.value.get_or_init(self.compute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_property() {
        fn compute_fn() -> i32 { 42 }
        let cp = CachedProperty::new(compute_fn);
        assert_eq!(*cp.get(), 42);
        assert_eq!(*cp.get(), 42); // cached, returns same value
    }

    #[test]
    fn test_memoize() {
        let f = memoize(|x: i32| x * 2);
        assert_eq!(f(5), 10);
        assert_eq!(f(5), 10); // cached
        assert_eq!(f(7), 14);
    }

    #[test]
    fn test_partition() {
        let items = [1, 2, 3, 4, 5];
        let (evens, odds) = partition(&items, |x| *x % 2 == 0);
        assert_eq!(evens, vec![&2, &4]);
        assert_eq!(odds, vec![&1, &3, &5]);
    }
}
