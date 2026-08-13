use std::sync::Arc;

use rustc_hash::FxHashMap;

#[derive(Default, Clone)]
pub(crate) struct ReferenceKeyCache {
    class_keys: FxHashMap<Arc<str>, Arc<str>>,
    function_keys: FxHashMap<Arc<str>, Arc<str>>,
    global_constant_keys: FxHashMap<Arc<str>, Arc<str>>,
    dynamic_keys: FxHashMap<Arc<str>, Arc<str>>,
    impl_keys: FxHashMap<Arc<str>, Arc<str>>,
    implshort_keys: FxHashMap<Arc<str>, Arc<str>>,
    propname_keys: FxHashMap<Arc<str>, Arc<str>>,
    propdecl_keys: FxHashMap<Arc<str>, Arc<str>>,
    cnstdecl_keys: FxHashMap<Arc<str>, Arc<str>>,
    method_keys: FxHashMap<Arc<str>, FxHashMap<Arc<str>, Arc<str>>>,
    property_keys: FxHashMap<Arc<str>, FxHashMap<Arc<str>, Arc<str>>>,
    traituse_keys: FxHashMap<Arc<str>, FxHashMap<Arc<str>, Arc<str>>>,
}

impl ReferenceKeyCache {
    pub(crate) fn class(&mut self, fqcn: &str) -> Arc<str> {
        cached_prefixed(&mut self.class_keys, "cls:", fqcn)
    }

    pub(crate) fn function(&mut self, fqn: &str) -> Arc<str> {
        cached_prefixed(&mut self.function_keys, "fn:", fqn)
    }

    pub(crate) fn global_constant(&mut self, fqn: &str) -> Arc<str> {
        cached_prefixed(&mut self.global_constant_keys, "gcnst:", fqn)
    }

    pub(crate) fn dynamic(&mut self, fqcn: &str) -> Arc<str> {
        cached_prefixed(&mut self.dynamic_keys, "dyn:", fqcn)
    }

    pub(crate) fn implementation(&mut self, fqcn: &str) -> Arc<str> {
        cached_prefixed(&mut self.impl_keys, "impl:", fqcn)
    }

    pub(crate) fn implementation_short(&mut self, short_name: &str) -> Arc<str> {
        cached_prefixed(&mut self.implshort_keys, "implshort:", short_name)
    }

    pub(crate) fn propname(&mut self, name: &str) -> Arc<str> {
        cached_prefixed(&mut self.propname_keys, "propname:", name)
    }

    pub(crate) fn propdecl(&mut self, name: &str) -> Arc<str> {
        cached_prefixed(&mut self.propdecl_keys, "propdecl:", name)
    }

    pub(crate) fn constant_decl(&mut self, name: &str) -> Arc<str> {
        cached_prefixed(&mut self.cnstdecl_keys, "cnstdecl:", name)
    }

    pub(crate) fn method(&mut self, class: &str, name: &str) -> Arc<str> {
        cached_pair(&mut self.method_keys, "meth:", class, name)
    }

    pub(crate) fn property(&mut self, class: &str, name: &str) -> Arc<str> {
        cached_pair(&mut self.property_keys, "prop:", class, name)
    }

    pub(crate) fn trait_use_property(&mut self, class: &str, name: &str) -> Arc<str> {
        cached_pair(&mut self.traituse_keys, "traituse:", class, name)
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

fn cached_pair(
    map: &mut FxHashMap<Arc<str>, FxHashMap<Arc<str>, Arc<str>>>,
    prefix: &str,
    left: &str,
    right: &str,
) -> Arc<str> {
    if let Some(inner) = map.get(left) {
        if let Some(key) = inner.get(right) {
            return key.clone();
        }
    }
    let left_arc: Arc<str> = Arc::from(left);
    let right_arc: Arc<str> = Arc::from(right);
    let key: Arc<str> = Arc::from(format!("{prefix}{left}::{right}"));
    map.entry(left_arc).or_default().insert(right_arc, key.clone());
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

    #[test]
    fn reuses_cached_pair_keys() {
        let mut cache = ReferenceKeyCache::default();
        let first = cache.property("App\\User", "name");
        let second = cache.property("App\\User", "name");
        assert!(std::sync::Arc::ptr_eq(&first, &second));
    }
}
