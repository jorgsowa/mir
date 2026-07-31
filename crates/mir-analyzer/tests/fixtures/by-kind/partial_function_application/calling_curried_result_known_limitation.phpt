===description===
Neither the partial-application call nor the later `$addFive(10)` call gets
analyzed at all here — the placeholder's version-gate ParseError is a hard
parse error, which blanks out body-level analysis for the whole file (see
`placeholder_parse_error_suppresses_whole_file_analysis.phpt`). If mir ever
starts targeting PHP 8.6 in the parser, this fixture's job changes: it
should then start exercising the (currently unmodeled) runtime semantics —
the call should evaluate to a new Closure, not to `add`'s own return type —
and this locked-in diff is where that gap will first become visible.
===config===
suppress=UnusedVariable
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$addFive = add(?, 5);
$result = $addFive(10);
===expect===
ParseError@7:15-7:16: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
