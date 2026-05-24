===description===
plainStringNotFalsePositiveUndefinedClass
===file===
<?php
// This test demonstrates the fix for issue #5:
// A plain string literal should NOT emit UndefinedClass even if it
// looks like it could be a class name.

// Before fix: Would emit "UndefinedClass: Class NonExistentClassNameString does not exist"
// After fix: No error, because plain strings are not resolved as FQCN references

$data = ["trim", "strlen", "implode"];
$results = array_map("trim", ["  a  ", "  b  ", "  c  "]);

// Using string values in various contexts - none should emit false positive UndefinedClass
$callback = "processItem";
$className = "SomeClass";
$methodName = "execute";

// Strings are just values, not class references, so no false positives
echo $callback;
echo $className;
echo $methodName;
===expect===
