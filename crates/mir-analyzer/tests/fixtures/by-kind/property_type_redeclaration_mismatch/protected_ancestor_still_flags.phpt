===description===
Negative control — a PROTECTED (not private) ancestor property is inherited, so
a changed native type must still be flagged.
===file===
<?php
class A {
    protected int $x = 1;
}

class B extends A {
    protected string $x = 'hello';
}
===expect===
PropertyTypeRedeclarationMismatch@7:4-7:34: Type of B::$x must be int (as in parent class), string given
