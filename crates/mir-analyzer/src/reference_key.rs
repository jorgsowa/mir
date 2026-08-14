use std::sync::Arc;

use rustc_hash::FxHashMap;

#[derive(Default, Clone)]
pub(crate) struct ReferenceKeyCache {
    class_keys: FxHashMap<Arc<str>, Arc<str>>,
}

impl ReferenceKeyCache {
    pub(crate) fn class(&mut self, fqcn: &str) -> Arc<str> {
        cached_prefixed(&mut self.class_keys, "cls:", fqcn)
    }
}

fn cached_prefixed(map: &mut FxHashMap<Arc<str>, Arc<str>>, prefix: &str, value: &str) -> Arc<str> {
    if let Some(key) = map.get(value) {
        return key.clone();
    }
    let value_arc: Arc<str> = Arc::from(value);
    let key: Arc<str> = Arc::from(format!("{prefix}{value}"));
    map.insert(value_arc, key.clone());
    key
}

#[cfg(test)]
mod tests {
    use super::ReferenceKeyCache;

    #[test]
    fn reuses_cached_single_part_keys() {
        let mut cache = ReferenceKeyCache::default();
        let first = cache.class("App\\User");
        let second = cache.class("App\\User");
        assert!(std::sync::Arc::ptr_eq(&first, &second));
    }
}
