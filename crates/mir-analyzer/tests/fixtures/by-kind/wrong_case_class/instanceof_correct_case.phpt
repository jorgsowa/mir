===description===
Correct case class name in instanceof is not reported.
===config===
suppress=UnusedVariable
===file===
<?php
class MyException extends \Exception {}
$e = new MyException();
$result = $e instanceof MyException;
===expect===
