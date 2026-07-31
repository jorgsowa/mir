===description===
FP-I1: the `ldap` PECL extension (ldap_connect, LDAP_OPT_* constants, ...)
had no vendored stubs/ dir despite PhpStormStubsMap.php already listing
every entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedParam,UnusedVariable,MixedAssignment,MixedArgument,MissingReturnType
===file===
<?php

function connect(string $uri) {
    return ldap_connect($uri);
}

function protocolVersionConstant(): int {
    return LDAP_OPT_PROTOCOL_VERSION;
}
===expect===
