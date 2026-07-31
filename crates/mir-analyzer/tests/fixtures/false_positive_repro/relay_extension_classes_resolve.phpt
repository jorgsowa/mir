===description===
FP-I1: the `relay` PECL extension (Relay\Relay, Relay\Cluster, ...) had no
vendored stubs/ dir despite PhpStormStubsMap.php already listing every
entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php

use Relay\Relay;
use Relay\Exception;

function connect(string $host): Relay {
    return new Relay($host);
}

function handle(Exception $e): void {
    echo $e->getCode();
}
===expect===
