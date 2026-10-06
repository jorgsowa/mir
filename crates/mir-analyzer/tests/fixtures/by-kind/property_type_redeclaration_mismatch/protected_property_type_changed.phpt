===description===
Child changes type of protected parent property — same error applies regardless of visibility
===file===
<?php
class Base {
    protected float $ratio = 1.0;
}

class Derived extends Base {
    protected int $ratio = 1;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Derived::$ratio must be float (as in parent class), int given
}
