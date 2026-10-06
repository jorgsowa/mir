===description===
Child redeclares a non-static parent property as static — fatal PHP error
===file===
<?php
class A {
    public int $x = 0;
}

class B extends A {
    public static int $x = 1;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ StaticPropertyRedeclarationMismatch: Cannot redeclare non-static property A::$x as static B::$x
}
