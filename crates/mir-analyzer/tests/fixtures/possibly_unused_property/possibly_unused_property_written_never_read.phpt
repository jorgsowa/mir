===description===
possiblyUnusedPropertyWrittenNeverRead
===file===
<?php
final class A {
    /** @var string */
    public $foo = "hello";
}

$a = new A();
$a->foo = "bar";
===expect===
PossiblyUnusedProperty
===ignore===
TODO
