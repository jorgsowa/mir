===description===
Child redeclares a readonly parent property as non-readonly — fatal PHP error
===file===
<?php
class A {
    public readonly int $x;
    public function __construct(int $x) {
        $this->x = $x;
    }
}

class B extends A {
    public int $x;
//  ^^^^^^^^^^^^^^ ReadonlyPropertyRedeclarationMismatch: Cannot redeclare readonly property A::$x as non-readonly B::$x
    public function __construct(int $x) {
        $this->x = $x;
    }
}
