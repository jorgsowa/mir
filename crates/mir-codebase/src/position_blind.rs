//! Definition equality that ignores line/column; an unblanked position field only makes it stricter.

use std::sync::Arc;

use mir_types::Location;

use crate::definitions::{
    ClassDef, ConstantDef, EnumCaseDef, EnumDef, FunctionDef, InterfaceDef, MemberMap, MethodDef,
    PropertyDef, TraitDef,
};

/// Equality that treats definitions differing only in line/column as equal.
pub trait PositionBlindEq {
    fn eq_ignoring_positions(&self, other: &Self) -> bool;
}

/// `loc` with its span zeroed; the file is kept, as it identifies the declaration.
fn blank(loc: &Location) -> Location {
    Location::new(loc.file.clone(), 0, 0, 0, 0)
}

fn blank_opt(loc: &mut Option<Location>) {
    if let Some(l) = loc.as_mut() {
        *l = blank(l);
    }
}

fn blank_all(locs: &mut [Location]) {
    for l in locs {
        *l = blank(l);
    }
}

fn blank_trait_uses(uses: &mut [(Arc<str>, Location)]) {
    for (_, l) in uses {
        *l = blank(l);
    }
}

fn blank_methods(methods: &mut MemberMap<Arc<MethodDef>>) {
    for m in methods.values_mut() {
        if m.location.is_some() {
            *m = Arc::new(m.without_positions());
        }
    }
}

fn blank_properties(props: &mut MemberMap<PropertyDef>) {
    for p in props.values_mut() {
        blank_opt(&mut p.location);
    }
}

fn blank_constants(consts: &mut MemberMap<ConstantDef>) {
    for c in consts.values_mut() {
        blank_opt(&mut c.location);
    }
}

fn blank_cases(cases: &mut MemberMap<EnumCaseDef>) {
    for c in cases.values_mut() {
        blank_opt(&mut c.location);
    }
}

macro_rules! position_blind_via_blanking {
    ($($ty:ty),* $(,)?) => {$(
        impl PositionBlindEq for $ty {
            fn eq_ignoring_positions(&self, other: &Self) -> bool {
                self == other || self.without_positions() == other.without_positions()
            }
        }
    )*};
}

position_blind_via_blanking!(
    MethodDef,
    PropertyDef,
    ConstantDef,
    EnumCaseDef,
    FunctionDef,
    ClassDef,
    InterfaceDef,
    TraitDef,
    EnumDef,
);

impl MethodDef {
    fn without_positions(&self) -> Self {
        let mut m = self.clone();
        blank_opt(&mut m.location);
        m
    }
}

impl PropertyDef {
    fn without_positions(&self) -> Self {
        let mut p = self.clone();
        blank_opt(&mut p.location);
        p
    }
}

impl ConstantDef {
    fn without_positions(&self) -> Self {
        let mut c = self.clone();
        blank_opt(&mut c.location);
        c
    }
}

impl EnumCaseDef {
    fn without_positions(&self) -> Self {
        let mut c = self.clone();
        blank_opt(&mut c.location);
        c
    }
}

impl FunctionDef {
    fn without_positions(&self) -> Self {
        let mut f = self.clone();
        blank_opt(&mut f.location);
        f
    }
}

impl ClassDef {
    fn without_positions(&self) -> Self {
        let mut c = self.clone();
        blank_opt(&mut c.location);
        blank_all(&mut c.mixin_locations);
        blank_trait_uses(&mut c.trait_use_locations);
        for l in c.trait_alias_locations.values_mut() {
            *l = blank(l);
        }
        blank_methods(&mut c.own_methods);
        blank_properties(&mut c.own_properties);
        blank_constants(&mut c.own_constants);
        c
    }
}

impl InterfaceDef {
    fn without_positions(&self) -> Self {
        let mut i = self.clone();
        blank_opt(&mut i.location);
        blank_methods(&mut i.own_methods);
        blank_properties(&mut i.own_properties);
        blank_constants(&mut i.own_constants);
        i
    }
}

impl TraitDef {
    fn without_positions(&self) -> Self {
        let mut t = self.clone();
        blank_opt(&mut t.location);
        blank_trait_uses(&mut t.trait_use_locations);
        blank_methods(&mut t.own_methods);
        blank_properties(&mut t.own_properties);
        blank_constants(&mut t.own_constants);
        t
    }
}

impl EnumDef {
    fn without_positions(&self) -> Self {
        let mut e = self.clone();
        blank_opt(&mut e.location);
        blank_trait_uses(&mut e.trait_use_locations);
        blank_cases(&mut e.cases);
        blank_methods(&mut e.own_methods);
        blank_properties(&mut e.own_properties);
        blank_constants(&mut e.own_constants);
        e
    }
}

impl<T: PositionBlindEq> PositionBlindEq for Arc<T> {
    fn eq_ignoring_positions(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other) || (**self).eq_ignoring_positions(other)
    }
}

impl<T: PositionBlindEq> PositionBlindEq for Option<T> {
    fn eq_ignoring_positions(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(a), Some(b)) => a.eq_ignoring_positions(b),
            (None, None) => true,
            _ => false,
        }
    }
}

impl<T: PositionBlindEq> PositionBlindEq for Vec<T> {
    fn eq_ignoring_positions(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other)
                .all(|(a, b)| a.eq_ignoring_positions(b))
    }
}

impl<T: PositionBlindEq> PositionBlindEq for (Arc<str>, T) {
    fn eq_ignoring_positions(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1.eq_ignoring_positions(&other.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class_at(line: u32) -> ClassDef {
        let at = |l| Location::new(Arc::from("/a.php"), l, l + 2, 4, 5);
        ClassDef {
            fqcn: Arc::from("A"),
            location: Some(at(line)),
            mixin_locations: vec![at(line + 1)],
            ..ClassDef::default()
        }
    }

    #[test]
    fn moved_class_compares_equal() {
        let (a, b) = (class_at(3), class_at(9));
        assert_ne!(a, b);
        assert!(a.eq_ignoring_positions(&b));
    }

    #[test]
    fn signature_change_still_differs() {
        let a = class_at(3);
        let b = ClassDef {
            is_final: !a.is_final,
            ..class_at(9)
        };
        assert!(!a.eq_ignoring_positions(&b));
    }
}
