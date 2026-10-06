===description===
Calling a static method with wrong casing is reported.
===file===
<?php
class Logger {
    public static function logError(): void {}
}
Logger::LOGERROR();
//      ^^^^^^^^ WrongCaseMethod: Method name 'Logger::LOGERROR' has incorrect casing; use 'logError'
