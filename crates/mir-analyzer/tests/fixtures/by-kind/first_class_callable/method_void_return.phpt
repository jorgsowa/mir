===description===
P3: Method returning void produces Closure(): void, not Closure(): mixed.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

class Logger {
    public function log(string $message): void {}
}

$logger = new Logger();
$fn = $logger->log(...);
/** @mir-check $fn is Closure(string): void */
$_ = $fn;
===expect===
