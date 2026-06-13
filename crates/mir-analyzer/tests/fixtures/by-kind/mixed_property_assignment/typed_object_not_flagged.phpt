===description===
MixedPropertyAssignment does NOT fire when the object has a concrete type.
===file===
<?php
class Foo {
    public string $name = "hello";
}

$obj = new Foo();
$obj->name = "world";

===expect===
