===description===
A method not proven mutation-free may return something different on the second call, so no narrowing.
===file===
<?php
class Y { public function m(): int { return 1; } }
class Z {}
class X {
    public function get(): Y|Z { return new Y; }
}

function f(X $x): void {
    if ($x->get() instanceof Y) {
        $x->get()->m();
//      ^^^^^^^^^^^^^^ UndefinedMethod: Method Z::m() does not exist
    }
}
