===description===
parameter type hint using bare FQN without use statement produces no error
===file:Logger.php===
<?php
class Logger {
    public function log(string $msg): void { echo $msg; }
}
===file:App.php===
<?php
function write(\Logger $logger, string $msg): void {
    $logger->log($msg);
}
===expect===
