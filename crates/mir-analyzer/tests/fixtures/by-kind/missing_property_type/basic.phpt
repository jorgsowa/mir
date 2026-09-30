===description===
MissingPropertyType fires for class properties without a type declaration.
===file===
<?php
class User {
    public $name;
//  ^^^^^^^^^^^^ MissingPropertyType: Property User::$name has no type annotation
    public $age;
//  ^^^^^^^^^^^ MissingPropertyType: Property User::$age has no type annotation
}
===expect===
