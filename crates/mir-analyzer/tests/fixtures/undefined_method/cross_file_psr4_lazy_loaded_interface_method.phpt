===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Contracts/Logger.php===
<?php
namespace App\Contracts;

interface Logger {
    public function log(string $message): void;
}
===file:src/NullLogger.php===
<?php
namespace App;

use App\Contracts\Logger;

class NullLogger implements Logger {
    public function log(string $message): void {}
}
===file:Main.php===
<?php
function test(): void {
    $logger = new Worker();
    $logger->log('ready');
}

class Worker extends \App\NullLogger {}
===expect===
