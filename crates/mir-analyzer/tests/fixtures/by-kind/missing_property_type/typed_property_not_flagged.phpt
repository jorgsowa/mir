===description===
MissingPropertyType does NOT fire when all properties have type declarations.
===file===
<?php
class User {
//<^^^^^^^^^^^^ MissingConstructor: Class User has uninitialized properties but no constructor
    public string $name;
    public int $age;
}
===expect===
