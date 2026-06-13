===description===
Correct case class name in catch clause is not reported.
===config===
suppress=UnusedVariable
===file===
<?php
class AppException extends \RuntimeException {}
try {
    throw new AppException("err");
} catch (AppException $e) {
}
===expect===
