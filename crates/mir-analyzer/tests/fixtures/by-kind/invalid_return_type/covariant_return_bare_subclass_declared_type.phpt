===description===
Covariant template param must still be honored when the DECLARED return
type names a bare subclass (`class IntBox extends Box {}`, no own
`@template` at all) instead of the class that actually declares
`@template-covariant T` — the variance lookup used the declared type's
own-only template params, empty for a bare subclass, defaulting every
position to Invariant and rejecting a valid covariant subtype. A genuine
sibling-type mismatch must still be rejected.
===file===
<?php
/** @template-covariant T */
class Box {
    /** @return T */
    public function get(): mixed { return null; }
}
class IntBox extends Box {}

class Animal {}
class Cat extends Animal {}
class Dog extends Animal {}

/** @return IntBox<Animal> */
function makeOk(): IntBox {
    /** @var IntBox<Cat> $b */
    $b = new IntBox();
    return $b;
}

/** @return IntBox<Cat> */
function makeBad(): IntBox {
    /** @var IntBox<Dog> $b */
    $b = new IntBox();
    return $b;
}
===expect===
InvalidReturnType@24:4-24:14: Return type 'IntBox<Dog>' is not compatible with declared 'IntBox<Cat>'
