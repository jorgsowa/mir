===description===
Correct case class name in instanceof is not reported.
===file===
<?php
class MyException extends \Exception {}
$e = new MyException();
$result = $e instanceof MyException;
===expect===
