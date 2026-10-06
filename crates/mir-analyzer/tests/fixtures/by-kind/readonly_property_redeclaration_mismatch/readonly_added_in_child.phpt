===description===
Child redeclares a non-readonly parent property as readonly — fatal PHP error
===file===
<?php
class A {
    public int $x = 0;
}

class B extends A {
    public readonly int $x;
//  ^^^^^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyRedeclarationMismatch: Cannot redeclare non-readonly property A::$x as readonly B::$x
    public function __construct() {
        $this->x = 1;
    }
}
