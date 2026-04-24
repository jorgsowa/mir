===file:Logger.php===
<?php
interface Logger {
    public function log(string $msg): void;
    public function error(string $msg): void;
}
===file:ConsoleLogger.php===
<?php
class ConsoleLogger implements Logger {
    public function log(string $msg): void { echo $msg; }
    public function error(string $msg): void { echo $msg; }
}
===expect===
