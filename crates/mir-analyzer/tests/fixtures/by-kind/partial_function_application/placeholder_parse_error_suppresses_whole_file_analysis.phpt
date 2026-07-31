===description===
Important, easy-to-miss behavior: mir's parser always targets PHP 8.5
internally (`php_rs_parser::parse()` is never called with an explicit
higher version, decoupled from mir's own `--php-version` flag), so ANY
partial-application placeholder unconditionally raises a `VersionTooLow`
ParseError — and because that's a *hard* parse error (`Severity::Error`),
`is_hard_parse_error` makes mir skip body-level analysis for the ENTIRE
file (see `db/per_function.rs` and `db/scopes.rs`). A second, genuinely
wrong-typed argument in the very same call — which mir correctly flags as
InvalidArgument with no placeholder present — goes completely unreported
here. This is not a bug introduced by the `Arg::value: Option<Expr>`
migration; it's the pre-existing "a hard parse error blanks out the whole
file's diagnostics" policy, just newly reachable via PHP 8.6 syntax.
===config===
suppress=UnusedVariable
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$partial = add(?, "not-an-int");
===expect===
ParseError@7:15-7:16: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
