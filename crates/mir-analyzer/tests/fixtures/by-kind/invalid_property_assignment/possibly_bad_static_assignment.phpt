===description===
Possibly bad static assignment
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var string */
    public static $foo = "a";

    public function barBar(): void
    {
        self::$foo = rand(0, 1) ? 5 : "hello";
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $foo expects 'string', cannot assign '5|"hello"'
    }
}
===expect===
