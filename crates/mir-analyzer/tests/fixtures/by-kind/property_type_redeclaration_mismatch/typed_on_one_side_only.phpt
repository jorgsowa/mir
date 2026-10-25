===description===
A redeclared property is typed on both sides or neither; private parents and stub-declared parents are exempt.
===file===
<?php
class Base {
    public int $count = 0;
    protected $name;
//  ^^^^^^^^^^^^^^^ MissingPropertyType: Property Base::$name has no type annotation
    private int $secret = 0;

    public function __construct(public int $id = 0) {}
}

class Child extends Base {
    public $count = 0;
//  ^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Child::$count has no type annotation
//  ^^^^^^^^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Child::$count must be int (as in parent class)
    protected string $name = '';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Child::$name must be omitted to match the parent definition
    public $secret;
//  ^^^^^^^^^^^^^^ MissingPropertyType: Property Child::$secret has no type annotation
    public $id;
//  ^^^^^^^^^^ MissingPropertyType: Property Child::$id has no type annotation
//  ^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Child::$id must be int (as in parent class)
}

class Failure extends Exception {
    protected string $file = '';
}
