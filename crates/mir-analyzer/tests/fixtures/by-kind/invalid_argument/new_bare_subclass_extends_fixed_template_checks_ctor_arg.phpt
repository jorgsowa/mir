===description===
`new IntBox("hello")` on a bare subclass with `@extends Box<int>` checks
the constructor argument against the inherited fixed template binding —
infer_new_type_params/analyze_new never substituted
inherited_template_bindings into the constructor's own param types before
arg-checking, unlike every other template-consuming call site. A bad
constructor arg used to also corrupt a later `@return T` call's inferred
type instead of being caught here.
===config===
suppress=MissingPropertyType,MissingConstructor
===file===
<?php
/** @template T */
class Box {
    private $value;

    /** @param T $value */
    public function __construct($value) {
        $this->value = $value;
    }

    /** @return T */
    public function getValue() {
        return $this->value;
    }
}

/** @extends Box<int> */
class IntBox extends Box {}

function needsInt(int $x): int {
    return $x;
}

new IntBox("hello");

$b = new IntBox(5);
needsInt($b->getValue());
===expect===
InvalidArgument@24:11-24:18: Argument $value of IntBox::__construct() expects 'int', got '"hello"'
