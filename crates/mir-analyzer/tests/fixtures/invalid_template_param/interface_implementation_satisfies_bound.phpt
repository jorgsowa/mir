===description===
Class implementing interface satisfies template bound of that interface
===file===
<?php
interface Logger {}
class ConsoleLogger implements Logger {}

/**
 * @template T of Logger
 * @param T $logger
 */
function setupLogging($logger): void {
    echo get_class($logger);
}

$logger = new ConsoleLogger();
setupLogging($logger);
===expect===
