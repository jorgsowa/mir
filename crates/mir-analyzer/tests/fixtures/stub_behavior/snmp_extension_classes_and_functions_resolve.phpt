===description===
FP-I1: the `snmp` PECL extension (SNMP class, snmpget/snmpwalk functions,
...) had no vendored stubs/ dir despite PhpStormStubsMap.php already
listing every entry — same missing-stub root cause as the fixed C6 (ast).
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function query(string $host, string $community, string $oid) {
//       ^^^^^ MissingReturnType: Function query() has no return type annotation
    return snmpget($host, $community, $oid);
}

function handle(SNMPException $e): string {
    return $e->getMessage();
}
