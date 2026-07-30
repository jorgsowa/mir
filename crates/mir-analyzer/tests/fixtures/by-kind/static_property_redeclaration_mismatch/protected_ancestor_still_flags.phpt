===description===
Negative control — a PROTECTED (not private) ancestor property is inherited, so
flipping static-ness must still be flagged.
===file===
<?php
class A {
    protected static int $x = 0;
}

class B extends A {
    protected int $x = 1;
}
===expect===
StaticPropertyRedeclarationMismatch@7:4-7:25: Cannot redeclare static property A::$x as non-static B::$x
