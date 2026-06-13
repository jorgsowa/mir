===description===
MissingPropertyType fires for class properties without a type declaration.
===file===
<?php
class User {
    public $name;
    public $age;
}
===expect===
MissingPropertyType@3:5-3:17: Property User::$name has no type annotation
MissingPropertyType@4:5-4:16: Property User::$age has no type annotation
