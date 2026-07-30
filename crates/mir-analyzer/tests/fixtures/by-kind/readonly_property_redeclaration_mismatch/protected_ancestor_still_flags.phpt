===description===
Negative control — a PROTECTED (not private) ancestor property is inherited, so
flipping readonly-ness must still be flagged.
===file===
<?php
class A {
    protected readonly int $x;
    public function __construct(int $x) {
        $this->x = $x;
    }
}

class B extends A {
    protected int $x;
    public function __construct(int $x) {
        $this->x = $x;
    }
}
===expect===
ReadonlyPropertyRedeclarationMismatch@10:4-10:21: Cannot redeclare readonly property A::$x as non-readonly B::$x
