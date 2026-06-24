===description===
Child changes type of protected parent property — same error applies regardless of visibility
===file===
<?php
class Base {
    protected float $ratio = 1.0;
}

class Derived extends Base {
    protected int $ratio = 1;
}
===expect===
PropertyTypeRedeclarationMismatch@7:4-7:29: Type of Derived::$ratio must be float (as in parent class), int given
