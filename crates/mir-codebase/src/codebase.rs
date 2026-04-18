use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dashmap::{DashMap, DashSet};

/// Maps symbol key → { file_path → {(start_byte, end_byte)} }.
/// Used by `Codebase::symbol_reference_locations`.
type ReferenceLocations = DashMap<Arc<str>, HashMap<Arc<str>, HashSet<(u32, u32)>>>;

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

    /// Types of `@var`-annotated global variables, collected in Pass 1.
    /// Key: variable name without the `$` prefix.
    pub global_vars: DashMap<Arc<str>, Union>,
    /// Maps file path → variable names declared with `@var` in that file.
    /// Used by `remove_file_definitions` to purge stale entries on re-analysis.
    file_global_vars: DashMap<Arc<str>, Vec<Arc<str>>>,

    /// Global PHP constants (`const FOO = 1` / `define('FOO', 1)`) keyed by value.
    /// Separate from `constants` to provide a per-file reverse index used by
    /// `remove_file_definitions` and Pass 1 snapshot building.
    file_constants: DashMap<Arc<str>, Vec<Arc<str>>>,

    /// Methods referenced during Pass 2 — key format: `"ClassName::methodName"`.
    /// Used by the dead-code detector (M18).
    pub referenced_methods: DashSet<Arc<str>>,
    /// Properties referenced during Pass 2 — key format: `"ClassName::propName"`.
    pub referenced_properties: DashSet<Arc<str>>,
    /// Free functions referenced during Pass 2 — key: fully-qualified name.
    pub referenced_functions: DashSet<Arc<str>>,

    /// Maps symbol key → { file_path → {(start_byte, end_byte)} }.
    /// Key format mirrors referenced_methods / referenced_properties / referenced_functions.
    /// The inner HashMap groups all spans from the same file under a single key,
    /// avoiding Arc<str> duplication per span and enabling O(1) per-file cleanup.
    /// HashSet deduplicates spans from union receivers (e.g. Foo|Foo->method()).
    pub symbol_reference_locations: ReferenceLocations,
    /// Reverse index: file_path → unique symbol keys referenced in that file.
    /// Used by remove_file_definitions for O(1) cleanup without a full map scan.
    pub file_symbol_references: DashMap<Arc<str>, HashSet<Arc<str>>>,

    /// Maps every FQCN (class, interface, trait, enum, function) to the absolute
    /// path of the file that defines it. Populated during Pass 1.
    pub symbol_to_file: DashMap<Arc<str>, Arc<str>>,

    /// Lightweight FQCN index populated by `SymbolTable` before Pass 1.
    /// Enables O(1) "does this symbol exist?" checks before full definitions
    /// are available.
    pub known_symbols: DashSet<Arc<str>>,

    /// Per-file `use` alias maps: alias → FQCN.  Populated during Pass 1.
    ///
    /// Key: absolute file path (as `Arc<str>`).
    /// Value: map of `alias → fully-qualified class name`.
    ///
    /// Exposed as `pub` so that external consumers (e.g. `php-lsp`) can read
    /// import data that mir already collects, instead of reimplementing it.
    pub file_imports: DashMap<Arc<str>, std::collections::HashMap<String, String>>,
    /// Per-file current namespace (if any).  Populated during Pass 1.
    ///
    /// Key: absolute file path (as `Arc<str>`).
    /// Value: the declared namespace string (e.g. `"App\\Controller"`).
    ///
    /// Exposed as `pub` so that external consumers (e.g. `php-lsp`) can read
    /// namespace data that mir already collects, instead of reimplementing it.
    pub file_namespaces: DashMap<Arc<str>, String>,

    /// Whether finalize() has been called.
    finalized: std::sync::atomic::AtomicBool,
}

impl Codebase {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the finalization flag so that `finalize()` will run again.
    ///
    /// Use this when new class definitions have been added after an initial
    /// `finalize()` call (e.g., lazily loaded via PSR-4) and the inheritance
    /// graph needs to be rebuilt.
    pub fn invalidate_finalization(&self) {
        self.finalized
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    // -----------------------------------------------------------------------
    // Incremental: remove all definitions from a single file
    // -----------------------------------------------------------------------

    /// Remove all definitions and outgoing reference locations contributed by the given file.
    /// This clears classes, interfaces, traits, enums, functions, and constants
    /// whose defining file matches `file_path`, the file's import and namespace entries,
    /// and all entries in symbol_reference_locations that originated from this file.
    /// After calling this, `invalidate_finalization()` is called so the next `finalize()`
    /// rebuilds inheritance.
    pub fn remove_file_definitions(&self, file_path: &str) {
        // Collect all symbols defined in this file
        let symbols: Vec<Arc<str>> = self
            .symbol_to_file
            .iter()
            .filter(|entry| entry.value().as_ref() == file_path)
            .map(|entry| entry.key().clone())
            .collect();

        // Remove each symbol from its respective map and from symbol_to_file.
        // Constants are NOT in symbol_to_file; they are removed below via file_constants.
        for sym in &symbols {
            self.classes.remove(sym.as_ref());
            self.interfaces.remove(sym.as_ref());
            self.traits.remove(sym.as_ref());
            self.enums.remove(sym.as_ref());
            self.functions.remove(sym.as_ref());
            self.symbol_to_file.remove(sym.as_ref());
            self.known_symbols.remove(sym.as_ref());
        }

        // Remove file-level metadata
        self.file_imports.remove(file_path);
        self.file_namespaces.remove(file_path);

        // Remove @var-annotated global variables declared in this file
        if let Some((_, var_names)) = self.file_global_vars.remove(file_path) {
            for name in var_names {
                self.global_vars.remove(name.as_ref());
            }
        }

        // Remove file-level constants declared in this file
        if let Some((_, const_names)) = self.file_constants.remove(file_path) {
            for name in const_names {
                self.constants.remove(name.as_ref());
            }
        }

        // Remove reference locations contributed by this file.
        // Use the reverse index to avoid a full scan of all symbols.
        if let Some((_, symbol_keys)) = self.file_symbol_references.remove(file_path) {
            for key in symbol_keys {
                if let Some(mut locs) = self.symbol_reference_locations.get_mut(&key) {
                    locs.remove(file_path);
                }
            }
        }

        self.invalidate_finalization();
    }

    // -----------------------------------------------------------------------
    // Global variable registry
    // -----------------------------------------------------------------------

    /// Return all `@var`-annotated global variable names declared in `file`.
    /// Used to build per-file Pass 1 snapshots for the definition cache.
    pub fn file_global_vars_for_file(&self, file: &Arc<str>) -> Vec<Arc<str>> {
        self.file_global_vars
            .get(file.as_ref())
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Record an `@var`-annotated global variable type discovered in Pass 1.
    /// If the same variable is annotated in multiple files, the last write wins.
    pub fn register_global_var(&self, file: &Arc<str>, name: Arc<str>, ty: Union) {
        self.file_global_vars
            .entry(file.clone())
            .or_default()
            .push(name.clone());
        self.global_vars.insert(name, ty);
    }

    // -----------------------------------------------------------------------
    // Global constant registry
    // -----------------------------------------------------------------------

    /// Record a global PHP constant (`const FOO = 1` / `define('FOO', 1)`) discovered in Pass 1.
    pub fn register_constant(&self, file: &Arc<str>, name: Arc<str>, ty: Union) {
        self.file_constants
            .entry(file.clone())
            .or_default()
            .push(name.clone());
        self.constants.insert(name, ty);
    }

    /// Return all global constant names declared in `file`.
    /// Used to build per-file Pass 1 snapshots for the definition cache.
    pub fn file_constants_for_file(&self, file: &Arc<str>) -> Vec<Arc<str>> {
        self.file_constants
            .get(file.as_ref())
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    // -----------------------------------------------------------------------
    // Lookups
    // -----------------------------------------------------------------------

    /// Resolve a property, walking up the inheritance chain (parent classes and traits).
    pub fn get_property(
        &self,
        fqcn: &str,
        prop_name: &str,
    ) -> Option<crate::storage::PropertyStorage> {
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
            if let Some(m) = iface.own_methods.get(method_name).or_else(|| {
                iface
                    .own_methods
                    .iter()
                    .find(|(k, _)| k.as_ref().eq_ignore_ascii_case(method_name))
                    .map(|(_, v)| v)
            }) {
                return Some(m.clone());
            }
            // Traverse parent interfaces
            let parents = iface.all_parents.clone();
            for parent_fqcn in &parents {
                if let Some(parent_iface) = self.interfaces.get(parent_fqcn.as_ref()) {
                    if let Some(m) = parent_iface.own_methods.get(method_name).or_else(|| {
                        parent_iface
                            .own_methods
                            .iter()
                            .find(|(k, _)| k.as_ref().eq_ignore_ascii_case(method_name))
                            .map(|(_, v)| v)
                    }) {
                        return Some(m.clone());
                    }
                }
            }
        }
        // Check trait methods (when a variable is annotated with a trait type)
        if let Some(tr) = self.traits.get(fqcn) {
            if let Some(m) = tr.own_methods.get(method_name).or_else(|| {
                tr.own_methods
                    .iter()
                    .find(|(k, _)| k.as_ref().eq_ignore_ascii_case(method_name))
                    .map(|(_, v)| v)
            }) {
                return Some(m.clone());
            }
        }
        // Check enum methods
        if let Some(e) = self.enums.get(fqcn) {
            if let Some(m) = e.own_methods.get(method_name).or_else(|| {
                e.own_methods
                    .iter()
                    .find(|(k, _)| k.as_ref().eq_ignore_ascii_case(method_name))
                    .map(|(_, v)| v)
            }) {
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
            if (ancestor == "BackedEnum" || ancestor == "\\BackedEnum") && en.scalar_type.is_some()
            {
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
        self.classes.get(fqcn).is_some_and(|c| c.is_abstract)
    }

    /// Return the declared template params for `fqcn` (class or interface), or
    /// an empty vec if the type is not found or has no templates.
    pub fn get_class_template_params(&self, fqcn: &str) -> Vec<crate::storage::TemplateParam> {
        if let Some(cls) = self.classes.get(fqcn) {
            return cls.template_params.clone();
        }
        if let Some(iface) = self.interfaces.get(fqcn) {
            return iface.template_params.clone();
        }
        if let Some(tr) = self.traits.get(fqcn) {
            return tr.template_params.clone();
        }
        vec![]
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
                if let Some(c) = self.classes.get(fqcn) {
                    c.all_parents.clone()
                } else {
                    vec![]
                }
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
            let Some(cls) = self.classes.get(fqcn) else {
                return false;
            };
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
    // Definition location lookups
    // -----------------------------------------------------------------------

    /// Look up the definition location of any symbol (class, interface, trait, enum, function).
    /// Returns the file path and byte offsets.
    pub fn get_symbol_location(&self, fqcn: &str) -> Option<crate::storage::Location> {
        if let Some(cls) = self.classes.get(fqcn) {
            return cls.location.clone();
        }
        if let Some(iface) = self.interfaces.get(fqcn) {
            return iface.location.clone();
        }
        if let Some(tr) = self.traits.get(fqcn) {
            return tr.location.clone();
        }
        if let Some(en) = self.enums.get(fqcn) {
            return en.location.clone();
        }
        if let Some(func) = self.functions.get(fqcn) {
            return func.location.clone();
        }
        None
    }

    /// Look up the definition location of a class member (method, property, constant).
    pub fn get_member_location(
        &self,
        fqcn: &str,
        member_name: &str,
    ) -> Option<crate::storage::Location> {
        // Check methods
        if let Some(method) = self.get_method(fqcn, member_name) {
            return method.location.clone();
        }
        // Check properties
        if let Some(prop) = self.get_property(fqcn, member_name) {
            return prop.location.clone();
        }
        // Check class constants
        if let Some(cls) = self.classes.get(fqcn) {
            if let Some(c) = cls.own_constants.get(member_name) {
                return c.location.clone();
            }
        }
        // Check interface constants
        if let Some(iface) = self.interfaces.get(fqcn) {
            if let Some(c) = iface.own_constants.get(member_name) {
                return c.location.clone();
            }
        }
        // Check trait constants
        if let Some(tr) = self.traits.get(fqcn) {
            if let Some(c) = tr.own_constants.get(member_name) {
                return c.location.clone();
            }
        }
        // Check enum constants and cases
        if let Some(en) = self.enums.get(fqcn) {
            if let Some(c) = en.own_constants.get(member_name) {
                return c.location.clone();
            }
            if let Some(case) = en.cases.get(member_name) {
                return case.location.clone();
            }
        }
        None
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

    /// Record a method reference with its source location.
    /// Also updates the referenced_methods DashSet for dead-code detection.
    pub fn mark_method_referenced_at(
        &self,
        fqcn: &str,
        method_name: &str,
        file: Arc<str>,
        start: u32,
        end: u32,
    ) {
        let key: Arc<str> = Arc::from(format!("{}::{}", fqcn, method_name.to_lowercase()).as_str());
        self.referenced_methods.insert(key.clone());
        self.symbol_reference_locations
            .entry(key.clone())
            .or_default()
            .entry(file.clone())
            .or_default()
            .insert((start, end));
        self.file_symbol_references
            .entry(file)
            .or_default()
            .insert(key);
    }

    /// Record a property reference with its source location.
    /// Also updates the referenced_properties DashSet for dead-code detection.
    pub fn mark_property_referenced_at(
        &self,
        fqcn: &str,
        prop_name: &str,
        file: Arc<str>,
        start: u32,
        end: u32,
    ) {
        let key: Arc<str> = Arc::from(format!("{}::{}", fqcn, prop_name).as_str());
        self.referenced_properties.insert(key.clone());
        self.symbol_reference_locations
            .entry(key.clone())
            .or_default()
            .entry(file.clone())
            .or_default()
            .insert((start, end));
        self.file_symbol_references
            .entry(file)
            .or_default()
            .insert(key);
    }

    /// Record a function reference with its source location.
    /// Also updates the referenced_functions DashSet for dead-code detection.
    pub fn mark_function_referenced_at(&self, fqn: &str, file: Arc<str>, start: u32, end: u32) {
        let key: Arc<str> = Arc::from(fqn);
        self.referenced_functions.insert(key.clone());
        self.symbol_reference_locations
            .entry(key.clone())
            .or_default()
            .entry(file.clone())
            .or_default()
            .insert((start, end));
        self.file_symbol_references
            .entry(file)
            .or_default()
            .insert(key);
    }

    /// Record a class reference (e.g. `new Foo()`) with its source location.
    /// Does not update any dead-code DashSet — class instantiation tracking is
    /// separate from method/property/function dead-code detection.
    pub fn mark_class_referenced_at(&self, fqcn: &str, file: Arc<str>, start: u32, end: u32) {
        let key: Arc<str> = Arc::from(fqcn);
        self.symbol_reference_locations
            .entry(key.clone())
            .or_default()
            .entry(file.clone())
            .or_default()
            .insert((start, end));
        self.file_symbol_references
            .entry(file)
            .or_default()
            .insert(key);
    }

    /// Replay cached reference locations for a file into symbol_reference_locations
    /// and file_symbol_references. Called on cache hits to avoid re-running Pass 2
    /// just to rebuild the reference index.
    /// `locs` is a slice of `(symbol_key, start_byte, end_byte)` as stored in the cache.
    pub fn replay_reference_locations(&self, file: Arc<str>, locs: &[(String, u32, u32)]) {
        for (symbol_key, start, end) in locs {
            let key: Arc<str> = Arc::from(symbol_key.as_str());
            self.symbol_reference_locations
                .entry(key.clone())
                .or_default()
                .entry(file.clone())
                .or_default()
                .insert((*start, *end));
            self.file_symbol_references
                .entry(file.clone())
                .or_default()
                .insert(key);
        }
    }

    /// Return all reference locations for `symbol` as a flat `Vec<(file, start, end)>`.
    /// Returns an empty Vec if the symbol has no recorded references.
    pub fn get_reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u32)> {
        match self.symbol_reference_locations.get(symbol) {
            None => Vec::new(),
            Some(by_file) => by_file
                .iter()
                .flat_map(|(file, spans)| {
                    spans.iter().map(|&(start, end)| (file.clone(), start, end))
                })
                .collect(),
        }
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

        self.finalized
            .store(true, std::sync::atomic::Ordering::SeqCst);
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
                    // Interface methods are implicitly abstract — mark them so that
                    // ClassAnalyzer::check_interface_methods_implemented can detect
                    // a concrete class that fails to provide an implementation.
                    let mut m = method.clone();
                    m.is_abstract = true;
                    table.insert(name.clone(), m);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn arc(s: &str) -> Arc<str> {
        Arc::from(s)
    }

    #[test]
    fn method_referenced_at_groups_spans_by_file() {
        let cb = Codebase::new();
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 0, 5);
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 10, 15);
        cb.mark_method_referenced_at("Foo", "bar", arc("b.php"), 20, 25);

        let locs = cb.symbol_reference_locations.get("Foo::bar").unwrap();
        assert_eq!(locs.len(), 2, "two files, not three spans");
        assert!(locs[&arc("a.php")].contains(&(0, 5)));
        assert!(locs[&arc("a.php")].contains(&(10, 15)));
        assert_eq!(locs[&arc("a.php")].len(), 2);
        assert!(locs[&arc("b.php")].contains(&(20, 25)));
        assert!(
            cb.is_method_referenced("Foo", "bar"),
            "DashSet also updated"
        );
    }

    #[test]
    fn duplicate_spans_are_deduplicated() {
        let cb = Codebase::new();
        // Same call site recorded twice (e.g. union receiver Foo|Foo)
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 0, 5);
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 0, 5);

        let locs = cb.symbol_reference_locations.get("Foo::bar").unwrap();
        assert_eq!(locs[&arc("a.php")].len(), 1, "duplicate span deduplicated");
    }

    #[test]
    fn method_key_is_lowercased() {
        let cb = Codebase::new();
        cb.mark_method_referenced_at("Cls", "MyMethod", arc("f.php"), 0, 3);
        assert!(cb.symbol_reference_locations.contains_key("Cls::mymethod"));
    }

    #[test]
    fn property_referenced_at_records_location() {
        let cb = Codebase::new();
        cb.mark_property_referenced_at("Bar", "count", arc("x.php"), 5, 10);

        let locs = cb.symbol_reference_locations.get("Bar::count").unwrap();
        assert!(locs[&arc("x.php")].contains(&(5, 10)));
        assert!(cb.is_property_referenced("Bar", "count"));
    }

    #[test]
    fn function_referenced_at_records_location() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("my_fn", arc("a.php"), 10, 15);

        let locs = cb.symbol_reference_locations.get("my_fn").unwrap();
        assert!(locs[&arc("a.php")].contains(&(10, 15)));
        assert!(cb.is_function_referenced("my_fn"));
    }

    #[test]
    fn class_referenced_at_records_location() {
        let cb = Codebase::new();
        cb.mark_class_referenced_at("Foo", arc("a.php"), 5, 8);

        let locs = cb.symbol_reference_locations.get("Foo").unwrap();
        assert!(locs[&arc("a.php")].contains(&(5, 8)));
    }

    #[test]
    fn get_reference_locations_flattens_all_files() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 0, 5);
        cb.mark_function_referenced_at("fn1", arc("b.php"), 10, 15);

        let mut locs = cb.get_reference_locations("fn1");
        locs.sort_by_key(|(_, s, _)| *s);
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0], (arc("a.php"), 0, 5));
        assert_eq!(locs[1], (arc("b.php"), 10, 15));
    }

    #[test]
    fn replay_reference_locations_restores_index() {
        let cb = Codebase::new();
        let locs = vec![
            ("Foo::bar".to_string(), 0u32, 5u32),
            ("Foo::bar".to_string(), 10, 15),
            ("greet".to_string(), 20, 25),
        ];
        cb.replay_reference_locations(arc("a.php"), &locs);

        let bar_locs = cb.symbol_reference_locations.get("Foo::bar").unwrap();
        assert!(bar_locs[&arc("a.php")].contains(&(0, 5)));
        assert!(bar_locs[&arc("a.php")].contains(&(10, 15)));

        let greet_locs = cb.symbol_reference_locations.get("greet").unwrap();
        assert!(greet_locs[&arc("a.php")].contains(&(20, 25)));

        let keys = cb.file_symbol_references.get(&arc("a.php")).unwrap();
        assert!(keys.contains(&Arc::from("Foo::bar")));
        assert!(keys.contains(&Arc::from("greet")));
    }

    #[test]
    fn remove_file_clears_its_spans_only() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 0, 5);
        cb.mark_function_referenced_at("fn1", arc("b.php"), 10, 15);

        cb.remove_file_definitions("a.php");

        let locs = cb.symbol_reference_locations.get("fn1").unwrap();
        assert!(!locs.contains_key("a.php"), "a.php spans removed");
        assert!(
            locs[&arc("b.php")].contains(&(10, 15)),
            "b.php spans untouched"
        );
        assert!(!cb.file_symbol_references.contains_key("a.php"));
    }

    #[test]
    fn remove_file_does_not_affect_other_files() {
        let cb = Codebase::new();
        cb.mark_property_referenced_at("Cls", "prop", arc("x.php"), 1, 4);
        cb.mark_property_referenced_at("Cls", "prop", arc("y.php"), 7, 10);

        cb.remove_file_definitions("x.php");

        let locs = cb.symbol_reference_locations.get("Cls::prop").unwrap();
        assert!(!locs.contains_key("x.php"));
        assert!(locs[&arc("y.php")].contains(&(7, 10)));
    }

    #[test]
    fn remove_file_definitions_on_never_analyzed_file_is_noop() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 0, 5);

        // "ghost.php" was never analyzed — removing it must not panic or corrupt state.
        cb.remove_file_definitions("ghost.php");

        // Existing data must be untouched.
        let locs = cb.symbol_reference_locations.get("fn1").unwrap();
        assert!(locs[&arc("a.php")].contains(&(0, 5)));
        assert!(!cb.file_symbol_references.contains_key("ghost.php"));
    }

    #[test]
    fn replay_reference_locations_with_empty_list_is_noop() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 0, 5);

        // Replaying an empty list must not touch existing entries.
        cb.replay_reference_locations(arc("b.php"), &[]);

        assert!(
            !cb.file_symbol_references.contains_key("b.php"),
            "empty replay must not create a file_symbol_references entry"
        );
        let locs = cb.symbol_reference_locations.get("fn1").unwrap();
        assert!(
            locs[&arc("a.php")].contains(&(0, 5)),
            "existing spans untouched"
        );
    }

    #[test]
    fn replay_reference_locations_twice_does_not_duplicate_spans() {
        let cb = Codebase::new();
        let locs = vec![("fn1".to_string(), 0u32, 5u32)];

        cb.replay_reference_locations(arc("a.php"), &locs);
        cb.replay_reference_locations(arc("a.php"), &locs);

        let by_file = cb.symbol_reference_locations.get("fn1").unwrap();
        assert_eq!(
            by_file[&arc("a.php")].len(),
            1,
            "replaying the same location twice must not create duplicate spans"
        );
    }
}
