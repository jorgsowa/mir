===description===
Array map use method on inferable int
===config===
suppress=MissingClosureReturnType,UnusedVariable
===file===
<?php
$a = array_map(function ($i) { return $i->foo(); }, [1, 2, 3, 4]);
===expect===
MixedMethodCall@2:39-2:48: Method foo() called on mixed type
