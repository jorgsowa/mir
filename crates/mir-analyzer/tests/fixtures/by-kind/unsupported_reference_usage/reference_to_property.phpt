===description===
Reference to an object property does not fire UnsupportedReferenceUsage.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {
    public string $bar = "x";
}

$obj = new Foo();
$ref = &$obj->bar;

===expect===
