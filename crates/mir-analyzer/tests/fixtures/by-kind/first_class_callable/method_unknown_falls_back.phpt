===description===
P3: First-class callable on an unknown method falls back to untyped callable without
emitting a false positive or panic.
===config===
suppress=UnusedVariable
===file===
<?php

class MyClass {}

$obj = new MyClass();
$fn = $obj->undefined(...);
===expect===
