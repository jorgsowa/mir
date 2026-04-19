//! Visible-members query for autocomplete / LSP completions.
//!
//! Given a `Union` type, returns all methods, properties, and constants that
//! are visible on that type, walking the full inheritance hierarchy.

use std::sync::Arc;

use mir_types::{Atomic, Union};

use crate::codebase::Codebase;
use crate::storage::Visibility;

/// A single member visible on a type.
#[derive(Debug, Clone)]
pub struct MemberInfo {
    /// Member name (without `$` prefix for properties).
    pub name: Arc<str>,
    /// What kind of member this is.
    pub kind: MemberKind,
    /// The resolved type of this member (return type for methods, property type, constant type).
    pub ty: Option<Union>,
    /// Visibility (public/protected/private).
    pub visibility: Visibility,
    /// Whether this is a static member.
    pub is_static: bool,
    /// The FQCN of the class that declares this member.
    pub declaring_class: Arc<str>,
    /// Whether this member is deprecated.
    pub is_deprecated: bool,
    /// Method parameters (empty for properties/constants).
    pub params: Vec<crate::storage::FnParam>,
}

/// The kind of class member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemberKind {
    Method,
    Property,
    Constant,
    EnumCase,
}

impl Codebase {
    /// Return all members (methods, properties, constants) visible on the given type.
    ///
    /// Walks the full class hierarchy including parents, interfaces, traits, and enums.
    /// For union types, returns the union of members from all constituent types.
    pub fn visible_members(&self, ty: &Union) -> Vec<MemberInfo> {
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for atomic in &ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                self.collect_members_for_fqcn(fqcn, &mut result, &mut seen);
            }
        }

        result
    }

    /// Collect all visible members for a single FQCN.
    fn collect_members_for_fqcn(
        &self,
        fqcn: &str,
        out: &mut Vec<MemberInfo>,
        seen: &mut std::collections::HashSet<(String, MemberKind)>,
    ) {
        // --- Class ---
        if let Some(cls) = self.classes.get(fqcn) {
            // Own methods (highest priority — first in, wins via `seen` dedup)
            for (name, method) in &cls.own_methods {
                let key = (name.to_string(), MemberKind::Method);
                if seen.insert(key) {
                    out.push(MemberInfo {
                        name: name.clone(),
                        kind: MemberKind::Method,
                        ty: method.effective_return_type().cloned(),
                        visibility: method.visibility,
                        is_static: method.is_static,
                        declaring_class: method.fqcn.clone(),
                        is_deprecated: method.is_deprecated,
                        params: method.params.clone(),
                    });
                }
            }

            // Collect chain before dropping the DashMap guard.
            let own_traits = cls.traits.clone();
            let all_parents = cls.all_parents.clone();
            let cls_fqcn = cls.fqcn.clone();

            // Own properties and constants
            for (name, prop) in &cls.own_properties {
                let key = (name.to_string(), MemberKind::Property);
                if seen.insert(key) {
                    out.push(MemberInfo {
                        name: name.clone(),
                        kind: MemberKind::Property,
                        ty: prop.ty.clone().or_else(|| prop.inferred_ty.clone()),
                        visibility: prop.visibility,
                        is_static: prop.is_static,
                        declaring_class: cls_fqcn.clone(),
                        is_deprecated: false,
                        params: vec![],
                    });
                }
            }
            for (name, con) in &cls.own_constants {
                let key = (name.to_string(), MemberKind::Constant);
                if seen.insert(key) {
                    out.push(MemberInfo {
                        name: name.clone(),
                        kind: MemberKind::Constant,
                        ty: Some(con.ty.clone()),
                        visibility: con.visibility.unwrap_or(Visibility::Public),
                        is_static: true,
                        declaring_class: cls_fqcn.clone(),
                        is_deprecated: false,
                        params: vec![],
                    });
                }
            }
            drop(cls);

            // Own trait methods and properties
            for tr_fqcn in &own_traits {
                if let Some(tr) = self.traits.get(tr_fqcn.as_ref()) {
                    for (name, method) in &tr.own_methods {
                        let key = (name.to_string(), MemberKind::Method);
                        if seen.insert(key) {
                            out.push(MemberInfo {
                                name: name.clone(),
                                kind: MemberKind::Method,
                                ty: method.effective_return_type().cloned(),
                                visibility: method.visibility,
                                is_static: method.is_static,
                                declaring_class: method.fqcn.clone(),
                                is_deprecated: method.is_deprecated,
                                params: method.params.clone(),
                            });
                        }
                    }
                    for (name, prop) in &tr.own_properties {
                        let key = (name.to_string(), MemberKind::Property);
                        if seen.insert(key) {
                            out.push(MemberInfo {
                                name: name.clone(),
                                kind: MemberKind::Property,
                                ty: prop.ty.clone().or_else(|| prop.inferred_ty.clone()),
                                visibility: prop.visibility,
                                is_static: prop.is_static,
                                declaring_class: tr.fqcn.clone(),
                                is_deprecated: false,
                                params: vec![],
                            });
                        }
                    }
                }
            }

            // Ancestor classes, their traits, and interfaces from all_parents
            for ancestor_fqcn in &all_parents {
                if let Some(ancestor) = self.classes.get(ancestor_fqcn.as_ref()) {
                    for (name, method) in &ancestor.own_methods {
                        let key = (name.to_string(), MemberKind::Method);
                        if seen.insert(key) {
                            out.push(MemberInfo {
                                name: name.clone(),
                                kind: MemberKind::Method,
                                ty: method.effective_return_type().cloned(),
                                visibility: method.visibility,
                                is_static: method.is_static,
                                declaring_class: method.fqcn.clone(),
                                is_deprecated: method.is_deprecated,
                                params: method.params.clone(),
                            });
                        }
                    }
                    for (name, prop) in &ancestor.own_properties {
                        let key = (name.to_string(), MemberKind::Property);
                        if seen.insert(key) {
                            out.push(MemberInfo {
                                name: name.clone(),
                                kind: MemberKind::Property,
                                ty: prop.ty.clone().or_else(|| prop.inferred_ty.clone()),
                                visibility: prop.visibility,
                                is_static: prop.is_static,
                                declaring_class: ancestor.fqcn.clone(),
                                is_deprecated: false,
                                params: vec![],
                            });
                        }
                    }
                    for (name, con) in &ancestor.own_constants {
                        let key = (name.to_string(), MemberKind::Constant);
                        if seen.insert(key) {
                            out.push(MemberInfo {
                                name: name.clone(),
                                kind: MemberKind::Constant,
                                ty: Some(con.ty.clone()),
                                visibility: con.visibility.unwrap_or(Visibility::Public),
                                is_static: true,
                                declaring_class: ancestor.fqcn.clone(),
                                is_deprecated: false,
                                params: vec![],
                            });
                        }
                    }
                    let anc_traits = ancestor.traits.clone();
                    drop(ancestor);
                    for tr_fqcn in &anc_traits {
                        if let Some(tr) = self.traits.get(tr_fqcn.as_ref()) {
                            for (name, method) in &tr.own_methods {
                                let key = (name.to_string(), MemberKind::Method);
                                if seen.insert(key) {
                                    out.push(MemberInfo {
                                        name: name.clone(),
                                        kind: MemberKind::Method,
                                        ty: method.effective_return_type().cloned(),
                                        visibility: method.visibility,
                                        is_static: method.is_static,
                                        declaring_class: method.fqcn.clone(),
                                        is_deprecated: method.is_deprecated,
                                        params: method.params.clone(),
                                    });
                                }
                            }
                            for (name, prop) in &tr.own_properties {
                                let key = (name.to_string(), MemberKind::Property);
                                if seen.insert(key) {
                                    out.push(MemberInfo {
                                        name: name.clone(),
                                        kind: MemberKind::Property,
                                        ty: prop.ty.clone().or_else(|| prop.inferred_ty.clone()),
                                        visibility: prop.visibility,
                                        is_static: prop.is_static,
                                        declaring_class: tr.fqcn.clone(),
                                        is_deprecated: false,
                                        params: vec![],
                                    });
                                }
                            }
                        }
                    }
                } else if let Some(iface) = self.interfaces.get(ancestor_fqcn.as_ref()) {
                    for (name, method) in &iface.own_methods {
                        let key = (name.to_string(), MemberKind::Method);
                        if seen.insert(key) {
                            out.push(MemberInfo {
                                name: name.clone(),
                                kind: MemberKind::Method,
                                ty: method.effective_return_type().cloned(),
                                visibility: method.visibility,
                                is_static: method.is_static,
                                declaring_class: method.fqcn.clone(),
                                is_deprecated: method.is_deprecated,
                                params: method.params.clone(),
                            });
                        }
                    }
                    for (name, con) in &iface.own_constants {
                        let key = (name.to_string(), MemberKind::Constant);
                        if seen.insert(key) {
                            out.push(MemberInfo {
                                name: name.clone(),
                                kind: MemberKind::Constant,
                                ty: Some(con.ty.clone()),
                                visibility: con.visibility.unwrap_or(Visibility::Public),
                                is_static: true,
                                declaring_class: iface.fqcn.clone(),
                                is_deprecated: false,
                                params: vec![],
                            });
                        }
                    }
                }
                // Traits in all_parents are already covered via their owning class's .traits above.
            }

            return;
        }

        // --- Interface ---
        if let Some(iface) = self.interfaces.get(fqcn) {
            for (name, method) in &iface.own_methods {
                let key = (name.to_string(), MemberKind::Method);
                if seen.insert(key) {
                    out.push(MemberInfo {
                        name: name.clone(),
                        kind: MemberKind::Method,
                        ty: method.effective_return_type().cloned(),
                        visibility: method.visibility,
                        is_static: method.is_static,
                        declaring_class: method.fqcn.clone(),
                        is_deprecated: method.is_deprecated,
                        params: method.params.clone(),
                    });
                }
            }
            for (name, con) in &iface.own_constants {
                let key = (name.to_string(), MemberKind::Constant);
                if seen.insert(key) {
                    out.push(MemberInfo {
                        name: name.clone(),
                        kind: MemberKind::Constant,
                        ty: Some(con.ty.clone()),
                        visibility: con.visibility.unwrap_or(Visibility::Public),
                        is_static: true,
                        declaring_class: iface.fqcn.clone(),
                        is_deprecated: false,
                        params: vec![],
                    });
                }
            }
            let parents = iface.all_parents.clone();
            drop(iface);
            for parent_fqcn in &parents {
                // Recurse into parent interfaces
                self.collect_members_for_fqcn(parent_fqcn, out, seen);
            }
            return;
        }

        // --- Enum ---
        if let Some(en) = self.enums.get(fqcn) {
            // Enum cases
            for (name, case) in &en.cases {
                let key = (name.to_string(), MemberKind::EnumCase);
                if seen.insert(key) {
                    out.push(MemberInfo {
                        name: name.clone(),
                        kind: MemberKind::EnumCase,
                        ty: case.value.clone(),
                        visibility: Visibility::Public,
                        is_static: true,
                        declaring_class: en.fqcn.clone(),
                        is_deprecated: false,
                        params: vec![],
                    });
                }
            }
            // Enum methods
            for (name, method) in &en.own_methods {
                let key = (name.to_string(), MemberKind::Method);
                if seen.insert(key) {
                    out.push(MemberInfo {
                        name: name.clone(),
                        kind: MemberKind::Method,
                        ty: method.effective_return_type().cloned(),
                        visibility: method.visibility,
                        is_static: method.is_static,
                        declaring_class: method.fqcn.clone(),
                        is_deprecated: method.is_deprecated,
                        params: method.params.clone(),
                    });
                }
            }
            // Enum constants
            for (name, con) in &en.own_constants {
                let key = (name.to_string(), MemberKind::Constant);
                if seen.insert(key) {
                    out.push(MemberInfo {
                        name: name.clone(),
                        kind: MemberKind::Constant,
                        ty: Some(con.ty.clone()),
                        visibility: con.visibility.unwrap_or(Visibility::Public),
                        is_static: true,
                        declaring_class: en.fqcn.clone(),
                        is_deprecated: false,
                        params: vec![],
                    });
                }
            }
            return;
        }

        // --- Trait (rare: variable typed as a trait) ---
        if let Some(tr) = self.traits.get(fqcn) {
            for (name, method) in &tr.own_methods {
                let key = (name.to_string(), MemberKind::Method);
                if seen.insert(key) {
                    out.push(MemberInfo {
                        name: name.clone(),
                        kind: MemberKind::Method,
                        ty: method.effective_return_type().cloned(),
                        visibility: method.visibility,
                        is_static: method.is_static,
                        declaring_class: method.fqcn.clone(),
                        is_deprecated: method.is_deprecated,
                        params: method.params.clone(),
                    });
                }
            }
            for (name, prop) in &tr.own_properties {
                let key = (name.to_string(), MemberKind::Property);
                if seen.insert(key) {
                    out.push(MemberInfo {
                        name: name.clone(),
                        kind: MemberKind::Property,
                        ty: prop.ty.clone().or_else(|| prop.inferred_ty.clone()),
                        visibility: prop.visibility,
                        is_static: prop.is_static,
                        declaring_class: tr.fqcn.clone(),
                        is_deprecated: false,
                        params: vec![],
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::*;
    use indexmap::IndexMap;

    fn make_method(name: &str, fqcn: &str) -> MethodStorage {
        MethodStorage {
            name: Arc::from(name),
            fqcn: Arc::from(fqcn),
            params: vec![],
            return_type: Some(Union::single(Atomic::TString)),
            inferred_return_type: None,
            visibility: Visibility::Public,
            is_static: false,
            is_abstract: false,
            is_final: false,
            is_constructor: false,
            template_params: vec![],
            assertions: vec![],
            throws: vec![],
            is_deprecated: false,
            is_internal: false,
            is_pure: false,
            location: None,
        }
    }

    #[test]
    fn visible_members_includes_inherited() {
        let cb = Codebase::new();

        // Parent class with a method
        let mut parent_methods = IndexMap::new();
        parent_methods.insert(
            Arc::from("parentMethod"),
            Arc::new(make_method("parentMethod", "Parent")),
        );
        cb.classes.insert(
            Arc::from("Parent"),
            ClassStorage {
                fqcn: Arc::from("Parent"),
                short_name: Arc::from("Parent"),
                parent: None,
                interfaces: vec![],
                traits: vec![],
                own_methods: parent_methods,
                own_properties: IndexMap::new(),
                own_constants: IndexMap::new(),
                template_params: vec![],
                is_abstract: false,
                is_final: false,
                is_readonly: false,
                all_parents: vec![],
                is_deprecated: false,
                is_internal: false,
                location: None,
            },
        );

        // Child class with its own method
        let mut child_methods = IndexMap::new();
        child_methods.insert(
            Arc::from("childMethod"),
            Arc::new(make_method("childMethod", "Child")),
        );
        cb.classes.insert(
            Arc::from("Child"),
            ClassStorage {
                fqcn: Arc::from("Child"),
                short_name: Arc::from("Child"),
                parent: Some(Arc::from("Parent")),
                interfaces: vec![],
                traits: vec![],
                own_methods: child_methods,
                own_properties: IndexMap::new(),
                own_constants: IndexMap::new(),
                template_params: vec![],
                is_abstract: false,
                is_final: false,
                is_readonly: false,
                all_parents: vec![],
                is_deprecated: false,
                is_internal: false,
                location: None,
            },
        );

        cb.finalize();

        let ty = Union::single(Atomic::TNamedObject {
            fqcn: Arc::from("Child"),
            type_params: vec![],
        });
        let members = cb.visible_members(&ty);
        let names: Vec<&str> = members.iter().map(|m| m.name.as_ref()).collect();
        assert!(names.contains(&"childMethod"), "should have own method");
        assert!(
            names.contains(&"parentMethod"),
            "should have inherited method"
        );
    }

    #[test]
    fn visible_members_union_type() {
        let cb = Codebase::new();

        let mut a_methods = IndexMap::new();
        a_methods.insert(Arc::from("aMethod"), Arc::new(make_method("aMethod", "A")));
        cb.classes.insert(
            Arc::from("A"),
            ClassStorage {
                fqcn: Arc::from("A"),
                short_name: Arc::from("A"),
                parent: None,
                interfaces: vec![],
                traits: vec![],
                own_methods: a_methods,
                own_properties: IndexMap::new(),
                own_constants: IndexMap::new(),
                template_params: vec![],
                is_abstract: false,
                is_final: false,
                is_readonly: false,
                all_parents: vec![],
                is_deprecated: false,
                is_internal: false,
                location: None,
            },
        );

        let mut b_methods = IndexMap::new();
        b_methods.insert(Arc::from("bMethod"), Arc::new(make_method("bMethod", "B")));
        cb.classes.insert(
            Arc::from("B"),
            ClassStorage {
                fqcn: Arc::from("B"),
                short_name: Arc::from("B"),
                parent: None,
                interfaces: vec![],
                traits: vec![],
                own_methods: b_methods,
                own_properties: IndexMap::new(),
                own_constants: IndexMap::new(),
                template_params: vec![],
                is_abstract: false,
                is_final: false,
                is_readonly: false,
                all_parents: vec![],
                is_deprecated: false,
                is_internal: false,
                location: None,
            },
        );

        cb.finalize();

        let ty = Union::merge(
            &Union::single(Atomic::TNamedObject {
                fqcn: Arc::from("A"),
                type_params: vec![],
            }),
            &Union::single(Atomic::TNamedObject {
                fqcn: Arc::from("B"),
                type_params: vec![],
            }),
        );
        let members = cb.visible_members(&ty);
        let names: Vec<&str> = members.iter().map(|m| m.name.as_ref()).collect();
        assert!(names.contains(&"aMethod"));
        assert!(names.contains(&"bMethod"));
    }
}
