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
WrongCaseClass@5:10-5:22: Class name 'appexception' has incorrect casing; use 'AppException'
