===description===
Possibly unused property written never read
===file===
<?php
final class A {
    /** @var string */
    public $foo = "hello";
}

$a = new A();
$a->foo = "bar";
===expect===
