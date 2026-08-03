===description===
MissingPropertyType does NOT fire for a property with no native type hint
when a valid `@var` docblock already gives it an explicit type.
===file===
<?php
class User {
    /** @var string */
    public $name;

    /** @var int */
    public $age;
}
===expect===
