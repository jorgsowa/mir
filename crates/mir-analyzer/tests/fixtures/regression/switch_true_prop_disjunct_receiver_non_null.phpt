===description===
switch(true) fallthrough with a mixed instanceof/is_TYPE() OR-disjunct on
a property, and a pure property-instanceof OR-disjunct, both correctly
propagate the union's receiver-non-null fact into the case body — not
just the last label's own single-condition narrowing. A disjunct that
includes an `is_null($x->prop)` leaf must NOT propagate receiver
non-null (matching a bare `if (A || B)` of the same shape), regardless
of case order.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
interface Value {}
class Foo implements Value {}
class Bar implements Value {}
class Box {
    public ?Value $value = null;
    public function ping(): void {}
}

function pureInstanceofDisjunctUnion(?Box $x): void {
    switch (true) {
        case $x->value instanceof Foo:
        case $x->value instanceof Bar:
            $x->ping();
    }
}

function mixedDisjunctCaseOrderA(?Box $x): void {
    switch (true) {
        case is_null($x->value):
        case $x->value instanceof Foo:
            /** @mir-check $x is Box|null */
            $_ = $x;
    }
}

function mixedDisjunctCaseOrderB(?Box $x): void {
    switch (true) {
        case $x->value instanceof Foo:
        case is_null($x->value):
            /** @mir-check $x is Box|null */
            $_ = $x;
    }
}
===expect===
PossiblyNullPropertyFetch@20:21-20:30: Cannot access property $value on possibly null value
PossiblyNullPropertyFetch@30:21-30:30: Cannot access property $value on possibly null value
