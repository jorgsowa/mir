===description===
Calling a non-pure method on a parameter fires ImpureMethodCall in a
@psalm-external-mutation-free method.
===file===
<?php

class Logger {
    public function log(string $msg): void {
        error_log($msg);
    }
}

class Service {
    /** @psalm-external-mutation-free */
    public function process(Logger $logger, string $msg): void {
        $logger->log($msg);
    }
}
===expect===
ImpureMethodCall@12:8-12:26: Calling impure method log() in a pure or immutable context
