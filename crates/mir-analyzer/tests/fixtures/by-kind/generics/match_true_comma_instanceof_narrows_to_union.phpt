===description===
Comma-separated `match(true)` arm conditions are OR semantics (the arm fires
if ANY condition is true) — `$foo instanceof A, $foo instanceof B` must
narrow $foo to A|B for the arm body. Before this fix each instanceof was
applied in sequence (AND semantics), collapsing $foo to just the LAST
disjunct (B) instead of the true union.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
interface Foo {}
class A implements Foo {}
class B implements Foo {}

function bar(Foo $foo): void {
    match (true) {
        $foo instanceof A, $foo instanceof B => (function () use ($foo) {
            /** @mir-check $foo is A|B */
            $_ = 1;
        })(),
        default => null,
    };
}
===expect===
