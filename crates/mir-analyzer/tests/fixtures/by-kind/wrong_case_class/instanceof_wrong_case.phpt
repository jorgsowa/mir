===description===
Wrong case class name in instanceof is reported.
===config===
suppress=UnusedVariable
===file===
<?php
class MyException extends \Exception {}
$e = new MyException();
$result = $e instanceof myexception;
===expect===
WrongCaseClass@4:24-4:35: Class name 'myexception' has incorrect casing; use 'MyException'
