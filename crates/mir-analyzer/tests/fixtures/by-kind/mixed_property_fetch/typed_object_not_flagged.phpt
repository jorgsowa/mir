===description===
MixedPropertyFetch does NOT fire when the object has a concrete type.
===file===
<?php
class Foo {
    public string $name = "hello";
}

$obj = new Foo();
$x = $obj->name;

===expect===
