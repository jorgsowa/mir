===description===
`call/opaque_callback.rs`'s `file_callable_call_args` is the one code path
in this whole migration that is genuinely reachable with a real `Arg {
value: None }` in practice: unlike everything under `body_analysis`, it
does its own independent `php_rs_parser::parse()` call and is never gated
by `is_hard_parse_error` — it scans every file in the workspace for
statically-resolvable callables passed to plain function calls, regardless
of whether that file itself has a (e.g. PFA-triggered) hard parse error.

This fixture proves its argument-position counter stays correctly aligned
when an earlier argument is a `?` placeholder: `process(?, 'double')` in
the placeholder-containing file must still record `double`'s return type
at position 1 (matching `$cb`'s param index in `process`'s own signature),
not at position 0 or dropped entirely — verified by checking that
`process`'s own body, which resolves `$cb`'s concrete return type purely
from this cross-file fact, infers the correct element type for its
`array_map($cb, ...)` call.
===config===
suppress=UnusedVariable,UnusedParam,UnusedFunction
===file:lib.php===
<?php

function process(int $mode, callable $cb): array {
    $result = array_map($cb, [1, 2, 3]);
    /** @mir-check $result is non-empty-list<int> */
    return $result;
}

function double(int $x): int {
    return $x * 2;
}
===file:caller_with_placeholder.php===
<?php

$partial = process(?, 'double');
===expect===
caller_with_placeholder.php: ParseError@3:19-3:20: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
