===description===
`get_class($x) === Foo::class` (and `$x::class === Foo::class`) preserve
a generic receiver's own type params on the narrowed exact-class atom,
mirroring how `instanceof` narrowing already does — previously these
always built a bare, raw `Foo` atom, discarding `Foo<int>`'s own `int`.
===config===
suppress=UnusedVariable,MissingConstructor,MissingPropertyType
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @var T */
    private $value;

    /** @param T $value */
    public function __construct($value) {
        $this->value = $value;
    }

    /** @return T */
    public function get() {
        return $this->value;
    }
}

/** @param Box<int> $b */
function viaGetClass(Box $b): int {
    if (get_class($b) === Box::class) {
        return $b->get();
    }
    return 0;
}

/** @param Box<int> $b */
function viaDynamicClassConst(Box $b): int {
    if ($b::class === Box::class) {
        return $b->get();
    }
    return 0;
}
===expect===
