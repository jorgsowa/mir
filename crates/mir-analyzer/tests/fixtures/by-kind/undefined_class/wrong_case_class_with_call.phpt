===description===
Wrong case class in type hint is now reported as WrongCaseClass.
===config===
suppress=UnusedParam
===file===
<?php
class A {}
needsA(new A);
function needsA(a $x): void {}
//              ^ WrongCaseClass: Class name 'a' has incorrect casing; use 'A'
===expect===
