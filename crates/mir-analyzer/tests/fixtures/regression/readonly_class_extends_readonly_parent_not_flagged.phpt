===description===
Negative counterpart: a readonly class extending another readonly class (both
sides agree) is not flagged.
===file===
<?php
readonly class A {
    public function __construct(public int $x) {}
}

readonly class B extends A {
    public function __construct(public int $x, public int $y) {
        parent::__construct($x);
    }
}
===expect===
