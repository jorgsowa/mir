===description===
A method declared only on a later intersection member (directly, via an
inherited interface, or via a trait) resolves with that member's return type.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
interface A { public function a(): int; }
interface B { public function b(): string; }
interface C extends B {}
trait T { public function t(): float { return 1.0; } }
class WithT { use T; }

function direct(A&B $x): void {
    $r = $x->b();
    /** @mir-check $r is string */
    $_ = $r;
}

function reversed(B&A $x): void {
    $r = $x->a();
    /** @mir-check $r is int */
    $_ = $r;
}

function inherited(A&C $x): void {
    $r = $x->b();
    /** @mir-check $r is string */
    $_ = $r;
}

function viaTrait(A&WithT $x): void {
    $r = $x->t();
    /** @mir-check $r is float */
    $_ = $r;
}

function narrowed(A $x): void {
    if ($x instanceof B) {
        $r = $x->b();
        /** @mir-check $r is string */
        $_ = $r;
    }
}

function firstClassCallable(A&B $x): void {
    $f = $x->b(...);
    $_ = $f;
}
===expect===
