===description===
MissingPropertyType fires for class properties without a type declaration.
===file===
<?php
class User {
    public $name;
    public $age;
}
===expect===
MissingPropertyType@3:4-3:16: Property User::$name has no type annotation
MissingPropertyType@4:4-4:15: Property User::$age has no type annotation
