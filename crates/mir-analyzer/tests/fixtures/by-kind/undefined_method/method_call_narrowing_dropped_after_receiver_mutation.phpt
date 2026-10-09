===description===
An impure call on the receiver invalidates a narrowed mutation-free call result.
===file===
<?php
class Y { public function m(): int { return 1; } }
class Z {}
class X {
    private int $resets = 0;
    /** @psalm-mutation-free */
    public function get(): Y|Z { return new Y; }
    public function reset(): void { $this->resets++; }
}

function f(X $x): void {
    if ($x->get() instanceof Y) {
        $x->reset();
        $x->get()->m();
//      ^^^^^^^^^^^^^^ PossiblyUndefinedMethod: Method Z::m() might not exist
    }
}

function reassigned(X $x, X $other): void {
    if ($x->get() instanceof Y) {
        $x = $other;
        $x->get()->m();
//      ^^^^^^^^^^^^^^ PossiblyUndefinedMethod: Method Z::m() might not exist
    }
}
