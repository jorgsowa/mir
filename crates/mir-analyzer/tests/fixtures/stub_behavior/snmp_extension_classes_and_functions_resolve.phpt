===description===
FP-I1: the `snmp` PECL extension (SNMP class, snmpget/snmpwalk functions,
...) had no vendored stubs/ dir despite PhpStormStubsMap.php already
listing every entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedParam,UnusedVariable,MixedAssignment,MixedArgument
===file===
<?php

function query(string $host, string $community, string $oid) {
    return snmpget($host, $community, $oid);
}

function handle(SNMPException $e): string {
    return $e->getMessage();
}
===expect===
MissingReturnType@3:9-3:14: Function query() has no return type annotation
