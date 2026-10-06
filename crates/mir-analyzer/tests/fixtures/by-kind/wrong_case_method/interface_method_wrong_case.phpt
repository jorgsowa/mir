===description===
Wrong case method name defined in interface is reported.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//  ^^^^^^^^^^ WrongCaseMethod: Method name 'ConsoleLogger::LOGMESSAGE' has incorrect casing; use 'logMessage'
