===description===
Calling a static method with wrong casing is reported.
===file===
<?php
class Logger {
    public static function logError(): void {}
}
Logger::LOGERROR();
===expect===
WrongCaseMethod@5:9-5:17: Method name 'Logger::LOGERROR' has incorrect casing; use 'logError'
