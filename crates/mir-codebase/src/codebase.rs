use std::sync::Arc;

use dashmap::{DashMap, DashSet};

use crate::storage::{
    ClassStorage, EnumStorage, FunctionStorage, InterfaceStorage, MethodStorage, TraitStorage,
};
use mir_types::Union;

// ---------------------------------------------------------------------------
// Codebase — thread-safe global symbol registry
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Codebase {
    pub classes: DashMap<Arc<str>, ClassStorage>,
    pub interfaces: DashMap<Arc<str>, InterfaceStorage>,
    pub traits: DashMap<Arc<str>, TraitStorage>,
    pub enums: DashMap<Arc<str>, EnumStorage>,
    pub functions: DashMap<Arc<str>, FunctionStorage>,
    pub constants: DashMap<Arc<str>, Union>,

    /// Methods referenced during Pass 2 — key format: `"ClassName::methodName"`.
    /// Used by the dead-code detector (M18).
    pub referenced_methods: DashSet<Arc<str>>,
    /// Properties referenced during Pass 2 — key format: `"ClassName::propName"`.
    pub referenced_properties: DashSet<Arc<str>>,
    /// Free functions referenced during Pass 2 — key: fully-qualified name.
    pub referenced_functions: DashSet<Arc<str>>,

    /// Per-file `use` alias maps: alias → FQCN.  Populated during Pass 1.
    pub file_imports: DashMap<Arc<str>, std::collections::HashMap<String, String>>,
    /// Per-file current namespace (if any).  Populated during Pass 1.
    pub file_namespaces: DashMap<Arc<str>, String>,

    /// Whether finalize() has been called.
    finalized: std::sync::atomic::AtomicBool,
}

impl Codebase {
    pub fn new() -> Self {
        Self::default()
    }

    // -----------------------------------------------------------------------
    // Lookups
    // -----------------------------------------------------------------------

    /// Resolve a property, walking up the inheritance chain (parent classes and traits).
    pub fn get_property(&self, fqcn: &str, prop_name: &str) -> Option<crate::storage::PropertyStorage> {
        // Check direct class own_properties
        if let Some(cls) = self.classes.get(fqcn) {
            if let Some(p) = cls.own_properties.get(prop_name) {
                return Some(p.clone());
            }
        }

        // Walk all ancestors (collected during finalize)
        let all_parents = {
            if let Some(cls) = self.classes.get(fqcn) {
                cls.all_parents.clone()
            } else {
                return None;
            }
        };

        for ancestor_fqcn in &all_parents {
            if let Some(ancestor_cls) = self.classes.get(ancestor_fqcn.as_ref()) {
                if let Some(p) = ancestor_cls.own_properties.get(prop_name) {
                    return Some(p.clone());
                }
            }
        }

        // Check traits
        let trait_list = {
            if let Some(cls) = self.classes.get(fqcn) {
                cls.traits.clone()
            } else {
                vec![]
            }
        };
        for trait_fqcn in &trait_list {
            if let Some(tr) = self.traits.get(trait_fqcn.as_ref()) {
                if let Some(p) = tr.own_properties.get(prop_name) {
                    return Some(p.clone());
                }
            }
        }

        None
    }

    /// Resolve a method, walking up the inheritance chain.
    pub fn get_method(&self, fqcn: &str, method_name: &str) -> Option<MethodStorage> {
        // PHP method names are case-insensitive — normalize to lowercase for all lookups.
        let method_lower = method_name.to_lowercase();
        let method_name = method_lower.as_str();
        // Check class methods first
        if let Some(cls) = self.classes.get(fqcn) {
            if let Some(m) = cls.get_method(method_name) {
                return Some(m.clone());
            }
        }
        // Check interface methods (including parent interfaces via all_parents)
        if let Some(iface) = self.interfaces.get(fqcn) {
            if let Some(m) = iface.own_methods.get(method_name)
                .or_else(|| iface.own_methods.iter().find(|(k, _)| k.as_ref().eq_ignore_ascii_case(method_name)).map(|(_, v)| v))
            {
                return Some(m.clone());
            }
            // Traverse parent interfaces
            let parents = iface.all_parents.clone();
            for parent_fqcn in &parents {
                if let Some(parent_iface) = self.interfaces.get(parent_fqcn.as_ref()) {
                    if let Some(m) = parent_iface.own_methods.get(method_name)
                        .or_else(|| parent_iface.own_methods.iter().find(|(k, _)| k.as_ref().eq_ignore_ascii_case(method_name)).map(|(_, v)| v))
                    {
                        return Some(m.clone());
                    }
                }
            }
        }
        // Check trait methods (when a variable is annotated with a trait type)
        if let Some(tr) = self.traits.get(fqcn) {
            if let Some(m) = tr.own_methods.get(method_name)
                .or_else(|| tr.own_methods.iter().find(|(k, _)| k.as_ref().eq_ignore_ascii_case(method_name)).map(|(_, v)| v))
            {
                return Some(m.clone());
            }
        }
        // Check enum methods
        if let Some(e) = self.enums.get(fqcn) {
            if let Some(m) = e.own_methods.get(method_name)
                .or_else(|| e.own_methods.iter().find(|(k, _)| k.as_ref().eq_ignore_ascii_case(method_name)).map(|(_, v)| v))
            {
                return Some(m.clone());
            }
            // PHP 8.1 built-in enum methods: cases(), from(), tryFrom()
            if matches!(method_name, "cases" | "from" | "tryfrom") {
                return Some(crate::storage::MethodStorage {
                    fqcn: Arc::from(fqcn),
                    name: Arc::from(method_name),
                    params: vec![],
                    return_type: Some(mir_types::Union::mixed()),
                    inferred_return_type: None,
                    visibility: crate::storage::Visibility::Public,
                    is_static: true,
                    is_abstract: false,
                    is_constructor: false,
                    template_params: vec![],
                    assertions: vec![],
                    throws: vec![],
                    is_final: false,
                    is_internal: false,
                    is_pure: false,
                    is_deprecated: false,
                    location: None,
                });
            }
        }
        None
    }

    /// Returns true if `child` extends or implements `ancestor` (transitively).
    pub fn extends_or_implements(&self, child: &str, ancestor: &str) -> bool {
        if child == ancestor {
            return true;
        }
        if let Some(cls) = self.classes.get(child) {
            return cls.implements_or_extends(ancestor);
        }
        if let Some(iface) = self.interfaces.get(child) {
            return iface.all_parents.iter().any(|p| p.as_ref() == ancestor);
        }
        // Enum: backed enums implicitly implement BackedEnum (and UnitEnum);
        // pure enums implicitly implement UnitEnum.
        if let Some(en) = self.enums.get(child) {
            // Check explicitly declared interfaces (e.g. implements SomeInterface)
            if en.interfaces.iter().any(|i| i.as_ref() == ancestor) {
                return true;
            }
            // PHP built-in: every enum implements UnitEnum
            if ancestor == "UnitEnum" || ancestor == "\\UnitEnum" {
                return true;
            }
            // Backed enums implement BackedEnum
            if (ancestor == "BackedEnum" || ancestor == "\\BackedEnum") && en.scalar_type.is_some() {
                return true;
            }
        }
        false
    }

    /// Whether a class/interface/trait/enum with this FQCN exists.
    pub fn type_exists(&self, fqcn: &str) -> bool {
        self.classes.contains_key(fqcn)
            || self.interfaces.contains_key(fqcn)
            || self.traits.contains_key(fqcn)
            || self.enums.contains_key(fqcn)
    }

    pub fn function_exists(&self, fqn: &str) -> bool {
        self.functions.contains_key(fqn)
    }

    /// Returns true if the class is declared abstract.
    /// Used to suppress `UndefinedMethod` on abstract class receivers: the concrete
    /// subclass is expected to implement the method, matching Psalm errorLevel=3 behaviour.
    pub fn is_abstract_class(&self, fqcn: &str) -> bool {
        self.classes.get(fqcn).map_or(false, |c| c.is_abstract)
    }

    /// Returns true if the class (or any ancestor/trait) defines a `__get` magic method.
    /// Such classes allow arbitrary property access, suppressing UndefinedProperty.
    pub fn has_magic_get(&self, fqcn: &str) -> bool {
        if let Some(cls) = self.classes.get(fqcn) {
            if cls.own_methods.contains_key("__get") || cls.all_methods.contains_key("__get") {
                return true;
            }
            // Check traits
            let traits = cls.traits.clone();
            drop(cls);
            for tr in &traits {
                if let Some(t) = self.traits.get(tr.as_ref()) {
                    if t.own_methods.contains_key("__get") {
                        return true;
                    }
                }
            }
            // Check ancestors
            let all_parents = {
                if let Some(c) = self.classes.get(fqcn) { c.all_parents.clone() } else { vec![] }
            };
            for ancestor in &all_parents {
                if let Some(anc) = self.classes.get(ancestor.as_ref()) {
                    if anc.own_methods.contains_key("__get") {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Returns true if the class (or any of its ancestors) has a parent/interface/trait
    /// that is NOT present in the codebase.  Used to suppress `UndefinedMethod` false
    /// positives: if a method might be inherited from an unscanned external class we
    /// cannot confirm or deny its existence.
    ///
    /// We use the pre-computed `all_parents` list (built during finalization) rather
    /// than recursive DashMap lookups to avoid potential deadlocks.
    pub fn has_unknown_ancestor(&self, fqcn: &str) -> bool {
        // For interfaces: check whether any parent interface is unknown.
        if let Some(iface) = self.interfaces.get(fqcn) {
            let parents = iface.all_parents.clone();
            drop(iface);
            for p in &parents {
                if !self.type_exists(p.as_ref()) {
                    return true;
                }
            }
            return false;
        }

        // Clone the data we need so the DashMap ref is dropped before any further lookups.
        let (parent, interfaces, traits, all_parents) = {
            let Some(cls) = self.classes.get(fqcn) else { return false };
            (
                cls.parent.clone(),
                cls.interfaces.clone(),
                cls.traits.clone(),
                cls.all_parents.clone(),
            )
        };

        // Fast path: check direct parent/interfaces/traits
        if let Some(ref p) = parent {
            if !self.type_exists(p.as_ref()) {
                return true;
            }
        }
        for iface in &interfaces {
            if !self.type_exists(iface.as_ref()) {
                return true;
            }
        }
        for tr in &traits {
            if !self.type_exists(tr.as_ref()) {
                return true;
            }
        }

        // Also check the full ancestor chain (pre-computed during finalization)
        for ancestor in &all_parents {
            if !self.type_exists(ancestor.as_ref()) {
                return true;
            }
        }

        false
    }

    /// Resolve a short class/function name to its FQCN using the import table
    /// and namespace recorded for `file` during Pass 1.
    ///
    /// - Names already containing `\` (after stripping a leading `\`) are
    ///   returned as-is (already fully qualified).
    /// - `self`, `parent`, `static` are returned unchanged (caller handles them).
    pub fn resolve_class_name(&self, file: &str, name: &str) -> String {
        let name = name.trim_start_matches('\\');
        if name.is_empty() {
            return name.to_string();
        }
        // Fully qualified absolute paths start with '\' (already stripped above).
        // Names containing '\' but not starting with it may be:
        //   - Already-resolved FQCNs (e.g. Frontify\Util\Foo) — check type_exists
        //   - Qualified relative names (e.g. Option\Some from within Frontify\Utility) — need namespace prefix
        if name.contains('\\') {
            // Check if the leading segment matches a use-import alias
            let first_segment = name.split('\\').next().unwrap_or(name);
            if let Some(imports) = self.file_imports.get(file) {
                if let Some(resolved_prefix) = imports.get(first_segment) {
                    let rest = &name[first_segment.len()..]; // includes leading '\'
                    return format!("{}{}", resolved_prefix, rest);
                }
            }
            // If already known in codebase as-is, it's FQCN — trust it
            if self.type_exists(name) {
                return name.to_string();
            }
            // Otherwise it's a relative qualified name — prepend the file namespace
            if let Some(ns) = self.file_namespaces.get(file) {
                let qualified = format!("{}\\{}", *ns, name);
                if self.type_exists(&qualified) {
                    return qualified;
                }
            }
            return name.to_string();
        }
        // Built-in pseudo-types / keywords handled by the caller
        match name {
            "self" | "parent" | "static" | "this" => return name.to_string(),
            _ => {}
        }
        // Check use aliases for this file (PHP class names are case-insensitive)
        if let Some(imports) = self.file_imports.get(file) {
            if let Some(resolved) = imports.get(name) {
                return resolved.clone();
            }
            // Fall back to case-insensitive alias lookup
            let name_lower = name.to_lowercase();
            for (alias, resolved) in imports.iter() {
                if alias.to_lowercase() == name_lower {
                    return resolved.clone();
                }
            }
        }
        // Qualify with the file's namespace if one exists
        if let Some(ns) = self.file_namespaces.get(file) {
            let qualified = format!("{}\\{}", *ns, name);
            // If the namespaced version exists in the codebase, use it.
            // Otherwise fall back to the global (unqualified) name if that exists.
            // This handles `DateTimeInterface`, `Exception`, etc. used without import
            // while not overriding user-defined classes in namespaces.
            if self.type_exists(&qualified) {
                return qualified;
            }
            if self.type_exists(name) {
                return name.to_string();
            }
            return qualified;
        }
        name.to_string()
    }

    // -----------------------------------------------------------------------
    // Reference tracking (M18 dead-code detection)
    // -----------------------------------------------------------------------

    /// Mark a method as referenced from user code.
    pub fn mark_method_referenced(&self, fqcn: &str, method_name: &str) {
        let key: Arc<str> = Arc::from(format!("{}::{}", fqcn, method_name.to_lowercase()).as_str());
        self.referenced_methods.insert(key);
    }

    /// Mark a property as referenced from user code.
    pub fn mark_property_referenced(&self, fqcn: &str, prop_name: &str) {
        let key: Arc<str> = Arc::from(format!("{}::{}", fqcn, prop_name).as_str());
        self.referenced_properties.insert(key);
    }

    /// Mark a free function as referenced from user code.
    pub fn mark_function_referenced(&self, fqn: &str) {
        self.referenced_functions.insert(Arc::from(fqn));
    }

    pub fn is_method_referenced(&self, fqcn: &str, method_name: &str) -> bool {
        let key = format!("{}::{}", fqcn, method_name.to_lowercase());
        self.referenced_methods.contains(key.as_str())
    }

    pub fn is_property_referenced(&self, fqcn: &str, prop_name: &str) -> bool {
        let key = format!("{}::{}", fqcn, prop_name);
        self.referenced_properties.contains(key.as_str())
    }

    pub fn is_function_referenced(&self, fqn: &str) -> bool {
        self.referenced_functions.contains(fqn)
    }

    // -----------------------------------------------------------------------
    // Finalization
    // -----------------------------------------------------------------------

    /// Must be called after all files have been parsed (pass 1 complete).
    /// Resolves inheritance chains and builds method dispatch tables.
    pub fn finalize(&self) {
        if self.finalized.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }

        // 1. Resolve all_parents for classes
        let class_keys: Vec<Arc<str>> = self.classes.iter().map(|e| e.key().clone()).collect();
        for fqcn in &class_keys {
            let parents = self.collect_class_ancestors(fqcn);
            if let Some(mut cls) = self.classes.get_mut(fqcn.as_ref()) {
                cls.all_parents = parents;
            }
        }

        // 2. Build method dispatch tables for classes (own methods override inherited)
        for fqcn in &class_keys {
            let all_methods = self.build_method_table(fqcn);
            if let Some(mut cls) = self.classes.get_mut(fqcn.as_ref()) {
                cls.all_methods = all_methods;
            }
        }

        // 3. Resolve all_parents for interfaces
        let iface_keys: Vec<Arc<str>> = self.interfaces.iter().map(|e| e.key().clone()).collect();
        for fqcn in &iface_keys {
            let parents = self.collect_interface_ancestors(fqcn);
            if let Some(mut iface) = self.interfaces.get_mut(fqcn.as_ref()) {
                iface.all_parents = parents;
            }
        }

        self.finalized.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn collect_class_ancestors(&self, fqcn: &str) -> Vec<Arc<str>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_class_ancestors_inner(fqcn, &mut result, &mut visited);
        result
    }

    fn collect_class_ancestors_inner(
        &self,
        fqcn: &str,
        out: &mut Vec<Arc<str>>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if !visited.insert(fqcn.to_string()) {
            return; // cycle guard
        }
        let (parent, interfaces, traits) = {
            if let Some(cls) = self.classes.get(fqcn) {
                (
                    cls.parent.clone(),
                    cls.interfaces.clone(),
                    cls.traits.clone(),
                )
            } else {
                return;
            }
        };

        if let Some(p) = parent {
            out.push(p.clone());
            self.collect_class_ancestors_inner(&p, out, visited);
        }
        for iface in interfaces {
            out.push(iface.clone());
            self.collect_interface_ancestors_inner(&iface, out, visited);
        }
        for t in traits {
            out.push(t);
        }
    }

    fn collect_interface_ancestors(&self, fqcn: &str) -> Vec<Arc<str>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_interface_ancestors_inner(fqcn, &mut result, &mut visited);
        result
    }

    fn collect_interface_ancestors_inner(
        &self,
        fqcn: &str,
        out: &mut Vec<Arc<str>>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if !visited.insert(fqcn.to_string()) {
            return;
        }
        let extends = {
            if let Some(iface) = self.interfaces.get(fqcn) {
                iface.extends.clone()
            } else {
                return;
            }
        };
        for e in extends {
            out.push(e.clone());
            self.collect_interface_ancestors_inner(&e, out, visited);
        }
    }

    /// Build the full method dispatch table for a class, with own methods taking
    /// priority over inherited ones.
    fn build_method_table(&self, fqcn: &str) -> indexmap::IndexMap<Arc<str>, MethodStorage> {
        use indexmap::IndexMap;
        let mut table: IndexMap<Arc<str>, MethodStorage> = IndexMap::new();

        // Walk ancestor chain (broad-first from root → child, so child overrides root)
        let ancestors = {
            if let Some(cls) = self.classes.get(fqcn) {
                cls.all_parents.clone()
            } else {
                return table;
            }
        };

        // Insert ancestor methods (deepest ancestor first, so closer ancestors override).
        // Also insert trait methods from ancestor classes.
        for ancestor_fqcn in ancestors.iter().rev() {
            if let Some(ancestor) = self.classes.get(ancestor_fqcn.as_ref()) {
                // First insert ancestor's own trait methods (lower priority)
                let ancestor_traits = ancestor.traits.clone();
                for trait_fqcn in ancestor_traits.iter().rev() {
                    if let Some(tr) = self.traits.get(trait_fqcn.as_ref()) {
                        for (name, method) in &tr.own_methods {
                            table.insert(name.clone(), method.clone());
                        }
                    }
                }
                // Then ancestor's own methods (override trait methods)
                for (name, method) in &ancestor.own_methods {
                    table.insert(name.clone(), method.clone());
                }
            } else if let Some(iface) = self.interfaces.get(ancestor_fqcn.as_ref()) {
                for (name, method) in &iface.own_methods {
                    table.insert(name.clone(), method.clone());
                }
            }
        }

        // Insert the class's own trait methods
        let trait_list = {
            if let Some(cls) = self.classes.get(fqcn) {
                cls.traits.clone()
            } else {
                vec![]
            }
        };
        for trait_fqcn in &trait_list {
            if let Some(tr) = self.traits.get(trait_fqcn.as_ref()) {
                for (name, method) in &tr.own_methods {
                    table.insert(name.clone(), method.clone());
                }
            }
        }

        // Own methods override everything
        if let Some(cls) = self.classes.get(fqcn) {
            for (name, method) in &cls.own_methods {
                table.insert(name.clone(), method.clone());
            }
        }

        table
    }
}
