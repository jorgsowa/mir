===description===
Array map use method on inferable int
===config===
suppress=MissingClosureReturnType,UnusedVariable
===file===
<?php
$a = array_map(function ($i) { return $i->foo(); }, [1, 2, 3, 4]);
//                                    ^^^^^^^^^ MixedMethodCall: Method foo() called on mixed type
===expect===
