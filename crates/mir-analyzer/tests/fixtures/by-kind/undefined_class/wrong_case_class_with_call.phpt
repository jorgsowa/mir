===description===
Wrong case class in type hint is now reported as WrongCaseClass.
===config===
suppress=UnusedParam
===file===
<?php
class A {}
needsA(new A);
function needsA(a $x): void {}
===expect===
InvalidArgument@3:8-3:13: Argument $x of needsA() expects 'a', got 'A'
WrongCaseClass@4:17-4:18: Class name 'a' has incorrect casing; use 'A'
