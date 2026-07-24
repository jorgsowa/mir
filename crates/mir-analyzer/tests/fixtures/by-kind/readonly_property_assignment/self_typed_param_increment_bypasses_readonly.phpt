===description===
`check_property_readonly_write` resolved the receiver via
`resolve_chained_receiver_type` (which deliberately does NOT rebind
self/static/parent, unlike the sibling cross-class immutable check right
above it) but only matched `Atomic::TNamedObject` — a `self`/`static`/
`parent`-typed parameter's `++`/`--`/`unset()`/array-index write on a
readonly property silently bypassed the check entirely, even though the
identical write through a concrete-class-typed parameter was already
caught.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
class Base {
    public readonly int $x;

    public function bumpSelfTyped(self $o): void {
        $o->x++;
    }

    /** @param static $o */
    public function bumpStaticTyped($o): void {
        $o->x++;
    }
}

class Sub extends Base {
    /** @param parent $o */
    public function bumpParentTyped($o): void {
        $o->x++;
    }
}
===expect===
ReadonlyPropertyAssignment@6:8-6:13: Cannot assign to readonly property Base::$x outside of constructor
ReadonlyPropertyAssignment@11:8-11:13: Cannot assign to readonly property Base::$x outside of constructor
ReadonlyPropertyAssignment@18:8-18:13: Cannot assign to readonly property Base::$x outside of constructor
