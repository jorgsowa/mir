===description===
Possibly unused property
===config===
suppress=MissingPropertyType,UnusedVariable
===file===
<?php
final class A {
    /** @var string */
    public $foo = "hello";
}

$a = new A();
===expect===
