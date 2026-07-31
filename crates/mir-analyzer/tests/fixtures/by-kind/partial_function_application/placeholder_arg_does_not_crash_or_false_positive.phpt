===description===
PHP 8.6 introduces partial function application (`?`/`...` call-argument
placeholders), parsed by php-rs-parser 0.19 as `Arg { value: None, .. }`.
mir's default target version is 8.5, so this correctly surfaces a
version-gate ParseError rather than crashing or silently misparsing — mir
doesn't model the resulting curried-closure semantics yet, but a
placeholder argument must not panic the analyzer or produce any diagnostic
beyond that single, accurate ParseError.
===config===
suppress=UnusedVariable
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$partial = add(?, 5);
===expect===
ParseError@7:15-7:16: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
