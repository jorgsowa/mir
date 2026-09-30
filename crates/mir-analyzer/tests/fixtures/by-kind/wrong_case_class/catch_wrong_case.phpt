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
//       ^^^^^^^^^^^^ WrongCaseClass: Class name 'appexception' has incorrect casing; use 'AppException'
}
===expect===
