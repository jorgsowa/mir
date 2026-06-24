===description===
Grandchild must match the nearest ancestor's typed property
===file===
<?php
class A {
    public int $x = 1;
}

class B extends A {
    public int $x = 2;
}

class C extends B {
    public string $x = 'wrong';
}
===expect===
PropertyTypeRedeclarationMismatch@11:4-11:31: Type of C::$x must be int (as in parent class), string given
