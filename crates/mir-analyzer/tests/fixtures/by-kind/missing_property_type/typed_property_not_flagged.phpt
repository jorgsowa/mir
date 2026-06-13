===description===
MissingPropertyType does NOT fire when all properties have type declarations.
===file===
<?php
class User {
    public string $name;
    public int $age;
}
===expect===
MissingConstructor@2:0-2:12: Class User has uninitialized properties but no constructor
