===description===
A get-only hook parent still requires the child type to be a subtype of the parent's.
===config===
php_version=8.4
===file===
<?php
abstract class Id {
    abstract public int|string $value { get; }
}

final class BadId extends Id {
    public float $value = 1.0;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of BadId::$value must be int|string (as in parent class), float given
}
===expect===
