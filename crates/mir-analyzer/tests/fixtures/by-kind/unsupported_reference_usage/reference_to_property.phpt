===description===
UnsupportedReferenceUsage fires when taking a reference to an object property.
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
UnsupportedReferenceUsage@7:1-7:18: Reference assignment is not supported
