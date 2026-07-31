===description===
Surprising but currently-real decoupling: the fixture's `php_version=8.6`
config only feeds mir's own `PhpVersion` (used for `@since`/`@removed` stub
filtering, e.g. via `db_php_version`) — it is never threaded into
`php_rs_parser::parse()`, which always targets its own internal default
(8.5) regardless. So targeting 8.6 explicitly does NOT silence the
partial-application version-gate ParseError today. If mir ever wires up
`parse_versioned` with its own configured version, this fixture's expected
output should change to empty — that'd be the signal the two version
systems were finally connected.
===config===
php_version=8.6
suppress=UnusedVariable
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$partial = add(?, 5);
===expect===
ParseError@7:15-7:16: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
