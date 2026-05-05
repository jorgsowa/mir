use dashmap::DashMap;
use std::sync::{Arc, LazyLock};

static INTERNER: LazyLock<DashMap<Arc<str>, ()>> = LazyLock::new(|| {
    let map = DashMap::with_capacity(1024);
    prewarm_php_strings(&map);
    map
});

pub fn intern(s: &str) -> Arc<str> {
    if let Some(entry) = INTERNER.get(s) {
        return entry.key().clone();
    }
    let arc: Arc<str> = Arc::from(s);
    INTERNER.entry(arc.clone()).or_insert(());
    arc
}

fn prewarm_php_strings(map: &DashMap<Arc<str>, ()>) {
    // Built-in PHP class names / interfaces referenced extensively across projects
    // Magic methods
    let strings = [
        "",
        "__construct",
        "__destruct",
        "__toString",
        "__invoke",
        "__get",
        "__set",
        "__isset",
        "__unset",
        "__call",
        "__callStatic",
        "__clone",
        "__debugInfo",
        "__serialize",
        "__unserialize",
        // Common PHP interfaces and base classes
        "stdClass",
        "Closure",
        "Generator",
        "Iterator",
        "IteratorAggregate",
        "Traversable",
        "ArrayAccess",
        "Countable",
        "Stringable",
        "Throwable",
        "Exception",
        "Error",
        "JsonSerializable",
        "Serializable",
        "DateTimeInterface",
        "DateTime",
        "DateTimeImmutable",
        "Fiber",
        "WeakReference",
        "WeakMap",
        "ArrayObject",
        "SplStack",
        "SplQueue",
        // Common namespaces
        "\\",
        "\\Exception",
        "\\stdClass",
        "\\Closure",
    ];

    for s in strings {
        let arc: Arc<str> = Arc::from(s);
        map.insert(arc, ());
    }
}
