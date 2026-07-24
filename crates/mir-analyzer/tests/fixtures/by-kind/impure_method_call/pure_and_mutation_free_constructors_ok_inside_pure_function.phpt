===description===
Contrast case for the constructor-purity check: a constructor explicitly
declared @pure, and one declared @mutation-free (which may still
initialize $this's own properties — not an external mutation), must both
stay unflagged when called via `new` inside a @pure function.
===config===
suppress=UnusedVariable
===file===
<?php
class PureBox {
    /** @pure */
    public function __construct(public int $v) {}
}

class MutationFreeBox {
    public int $v;

    /** @mutation-free */
    public function __construct(int $v) {
        $this->v = $v;
    }
}

/** @pure */
function make(): void {
    $p = new PureBox(1);
    $m = new MutationFreeBox(1);
}
===expect===
