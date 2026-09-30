===description===
Child widens parent int property to int|null — fatal PHP error
===file===
<?php
class A {
    public int $count = 0;
}

class B extends A {
    public ?int $count = null;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of B::$count must be int (as in parent class), int|null given
}
===expect===
