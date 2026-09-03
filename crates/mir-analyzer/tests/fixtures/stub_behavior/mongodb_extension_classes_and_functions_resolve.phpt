===description===
FP-I1: the `mongodb` PECL extension (MongoDB\Driver\Manager, BSON types,
exceptions) had no vendored stubs/ dir despite PhpStormStubsMap.php already
listing every entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedParam,UnusedVariable,MixedAssignment
===file===
<?php

use MongoDB\Driver\Manager;
use MongoDB\BSON\ObjectId;
use MongoDB\BSON\UTCDateTime;
use MongoDB\Driver\Exception\ConnectionException;

function connect(string $uri): Manager {
    return new Manager($uri);
}

function idAsString(ObjectId $id): string {
    return (string) $id;
}

function timestamp(): UTCDateTime {
    return new UTCDateTime();
}

function handle(ConnectionException $e): string {
    return $e->getMessage();
}
===expect===
