===description===
Wrong case class name in instanceof is reported.
===file===
<?php
class MyException extends \Exception {}
$e = new MyException();
$e instanceof myexception;
===expect===
WrongCaseClass@4:15-4:26: Class name 'myexception' has incorrect casing; use 'MyException'
