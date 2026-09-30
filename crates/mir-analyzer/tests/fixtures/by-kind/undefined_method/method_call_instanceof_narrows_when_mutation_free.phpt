===description===
`$x->get() instanceof Y` narrows later identical calls when the method is mutation-free.
===file===
<?php
class Y { public function m(): int { return 1; } }
class Z {}
class X {
    /** @psalm-mutation-free */
    public function get(): Y|Z { return new Y; }
}

function narrowed(X $x): void {
    if ($x->get() instanceof Y) {
        $r = $x->get();
        /** @mir-check $r is Y */
        echo gettype($r);
        $x->get()->m();
    }
}

function negated(X $x): void {
    if (!($x->get() instanceof Y)) {
        $r = $x->get();
        /** @mir-check $r is Z */
        echo gettype($r);
        return;
    }
    $r = $x->get();
    /** @mir-check $r is Y */
        echo gettype($r);
}
===expect===
