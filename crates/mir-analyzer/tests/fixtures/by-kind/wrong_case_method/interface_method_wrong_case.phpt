===description===
Wrong case method name defined in interface is reported.
===config===
suppress=UnusedParam
===file===
<?php
interface Logger {
    public function logMessage(string $msg): void;
}
class ConsoleLogger implements Logger {
    public function logMessage(string $msg): void {}
}
$l = new ConsoleLogger();
$l->LOGMESSAGE("hello");
===expect===
WrongCaseMethod@9:5-9:15: Method name 'ConsoleLogger::LOGMESSAGE' has incorrect casing; use 'logMessage'
