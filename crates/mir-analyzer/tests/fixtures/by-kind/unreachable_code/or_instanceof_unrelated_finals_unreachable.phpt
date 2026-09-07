===description===
`$x instanceof A || $x instanceof B` (and the property-receiver form) for a
value whose real type is an unrelated `final` class must be flagged as
unreachable — `narrow_or_instanceof_union` silently reset an empty (i.e.
provably-impossible) result to the bare `A|B` union instead of propagating
the emptiness, masking the contradiction and corrupting the narrowed type.
Uses the `@mir-check $_ is never` reachability probe.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
final class Baz {}
final class A {}
final class B {}

function orInstanceofUnrelatedFinalsUnreachable(Baz $x): void {
    if ($x instanceof A || $x instanceof B) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

class Holder {
    public Baz $prop;
}

function propOrInstanceofUnrelatedFinalsUnreachable(Holder $h): void {
    if ($h->prop instanceof A || $h->prop instanceof B) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

interface Quacks {}
class Duck implements Quacks {}
class Goose implements Quacks {}

function orInstanceofRelatedStillNarrows(Quacks $x): void {
    if ($x instanceof Duck || $x instanceof Goose) {
        /** @mir-check $x is Duck|Goose */
        $_ = 1;
    }
}
===expect===
RedundantCondition@7:8-7:42: Condition is always true/false for type 'bool'
RedundantCondition@18:8-18:54: Condition is always true/false for type 'bool'
