===description===
P3: First-class callable on an unknown method reports UndefinedMethod (like
the ordinary call form) instead of silently falling back to untyped callable.
===config===
suppress=UnusedVariable
===file===
<?php

class MyClass {}

$obj = new MyClass();
$fn = $obj->undefined(...);
===expect===
UndefinedMethod@6:12-6:21: Method MyClass::undefined() does not exist
