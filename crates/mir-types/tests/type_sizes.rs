use mir_types::{Atomic, Type};
use std::mem::size_of;

// Type is cloned/hashed on nearly every inference step and inlines two Atomics;
// these fail loudly when a new enum variant accidentally grows both.
#[test]
fn atomic_stays_small() {
    assert!(
        size_of::<Atomic>() <= 32,
        "Atomic grew to {} bytes",
        size_of::<Atomic>()
    );
}

#[test]
fn type_stays_small() {
    assert!(
        size_of::<Type>() <= 80,
        "Type grew to {} bytes",
        size_of::<Type>()
    );
}
