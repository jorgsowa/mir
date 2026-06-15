===description===
Wrong case class name in catch clause is reported.
===config===
suppress=UnusedVariable
===file===
<?php
class AppException extends \RuntimeException {}
try {
    throw new AppException("err");
} catch (appexception $e) {
}
===expect===
WrongCaseClass@5:9-5:21: Class name 'appexception' has incorrect casing; use 'AppException'
