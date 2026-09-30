===description===
A parent hook with a set accessor is backed storage; flipping to readonly is still an error
===file===
<?php
interface Settable {
    public int $x { get; set; }
}

class Base {
    public int $x = 0 { set { $this->x = $value; } }
}

class Child extends Base {
    public readonly int $x;
    public function __construct() {
        $this->x = 1;
    }
}
===expect===
ReadonlyPropertyRedeclarationMismatch@11:4-11:27: Cannot redeclare non-readonly property Base::$x as readonly Child::$x
