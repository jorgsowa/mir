===description===
Child redeclares a static parent property as non-static — fatal PHP error
===file===
<?php
class A {
    public static int $x = 0;
}

class B extends A {
    public int $x = 1;
}
===expect===
StaticPropertyRedeclarationMismatch@7:4-7:22: Cannot redeclare static property A::$x as non-static B::$x
