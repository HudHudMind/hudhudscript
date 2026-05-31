#[cfg(test)]
mod tests {
    use hudhudscript_bytecode::cache_utils::*;
    use std::collections::HashMap;

    #[test]
    fn test_enforce_cache_limit_under_limit() {
        let mut map = HashMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        let evicted = enforce_cache_limit(&mut map, 10);
        assert_eq!(evicted, 0);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_enforce_cache_limit_over_limit() {
        let mut map = HashMap::new();
        for i in 0..300 {
            map.insert(i, i);
        }
        let evicted = enforce_cache_limit(&mut map, 256);
        assert_eq!(evicted, 172); // 300 - 128
        assert_eq!(map.len(), 128); // 256 / 2
    }

    #[test]
    fn test_enforce_cache_limit_exactly_at_limit() {
        let mut map = HashMap::new();
        for i in 0..256 {
            map.insert(i, i);
        }
        let evicted = enforce_cache_limit(&mut map, 256);
        assert_eq!(evicted, 0);
        assert_eq!(map.len(), 256);
    }

    #[test]
    fn test_enforce_cache_limit_one_over() {
        let mut map = HashMap::new();
        for i in 0..257 {
            map.insert(i, i);
        }
        let evicted = enforce_cache_limit(&mut map, 256);
        assert_eq!(evicted, 129); // 257 - 128
        assert_eq!(map.len(), 128);
    }
}
