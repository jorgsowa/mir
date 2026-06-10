===description===
Possibly unused property
===ignore===
TODO
===file===
<?php
final class A {
    /** @var string */
    public $foo = "hello";
}

$a = new A();
===expect===
