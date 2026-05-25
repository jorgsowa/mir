===description===
Possibly unused property
===file===
<?php
final class A {
    /** @var string */
    public $foo = "hello";
}

$a = new A();
===expect===
PossiblyUnusedProperty
===ignore===
TODO
