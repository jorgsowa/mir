===description===
Child redeclares parent int property as string — fatal PHP error
===file===
<?php
class A {
    public int $x = 1;
}

class B extends A {
    public string $x = 'hello';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of B::$x must be int (as in parent class), string given
}
