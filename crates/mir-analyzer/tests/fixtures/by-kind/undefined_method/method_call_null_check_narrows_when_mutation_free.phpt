===description===
`$x->get() !== null` removes null from later identical calls when the method is mutation-free.
===file===
<?php
class Y {}
class X {
    /** @psalm-mutation-free */
    public function get(): ?Y { return null; }
}

function check(X $x): void {
    if ($x->get() !== null) {
        $r = $x->get();
        /** @mir-check $r is Y */
        echo gettype($r);
    }
    if ($x->get() === null) {
        return;
    }
    $s = $x->get();
    /** @mir-check $s is Y */
        echo gettype($s);
}
