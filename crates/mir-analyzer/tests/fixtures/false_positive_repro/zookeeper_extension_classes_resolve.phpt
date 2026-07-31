===description===
FP-I1: the `zookeeper` PECL extension (Zookeeper, ZookeeperException, ...)
had no vendored stubs/ dir despite PhpStormStubsMap.php already listing
every entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php

function connect(string $hosts): Zookeeper {
    return new Zookeeper($hosts);
}

function handle(ZookeeperConnectionException $e): string {
    return $e->getMessage();
}
===expect===
