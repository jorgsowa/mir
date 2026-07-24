===description===
A bare class reference (no explicit `<...>` type args at all, e.g. a plain
`new Box()` constructor call with nothing to bind T from) never fell back to
a class template's declared default (`@template T = Default`) — every
consumer of `build_class_bindings` left T entirely unbound, which downstream
treated as an unconstrained wildcard (mixed) instead of the declared
default.
===config===
suppress=UnusedParam,UnusedVariable,MissingConstructor,MissingPropertyType
===file===
<?php
/** @template T = int */
class Box {
    /** @var T */
    public $value;
}

function test(): void {
    $box = new Box();
    /** @mir-check $box->value is int */
    $_ = $box->value;
}
===expect===
