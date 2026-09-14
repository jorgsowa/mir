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
WrongCaseClass@4:16-4:17: Class name 'a' has incorrect casing; use 'A'
