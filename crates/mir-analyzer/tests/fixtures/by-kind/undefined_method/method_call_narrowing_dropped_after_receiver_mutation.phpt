===description===
An impure call on the receiver invalidates a narrowed mutation-free call result.
===file===
<?php
class Y { public function m(): int { return 1; } }
class Z {}
class X {
    /** @psalm-mutation-free */
    public function get(): Y|Z { return new Y; }
    public function reset(): void {}
}

function f(X $x): void {
    if ($x->get() instanceof Y) {
        $x->reset();
        $x->get()->m();
//      ^^^^^^^^^^^^^^ UndefinedMethod: Method Z::m() does not exist
    }
}

function reassigned(X $x, X $other): void {
    if ($x->get() instanceof Y) {
        $x = $other;
        $x->get()->m();
//      ^^^^^^^^^^^^^^ UndefinedMethod: Method Z::m() does not exist
    }
}
===expect===
