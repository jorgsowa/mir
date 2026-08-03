===description===
getenv()'s stub merges two arg-count-dependent overloads into one
array|string|false union. A non-null $name can only take the string|false
branch (never the whole-environment array); a bare or null $name can only
take the array|false branch. Only an unresolvable (e.g. nullable) $name
falls back to the stub's full union.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function test_no_arg(): void {
    $v = getenv();
    /** @mir-check $v is array<string, string>|false */
    $_ = $v;
}

function test_null_arg(): void {
    $v = getenv(null);
    /** @mir-check $v is array<string, string>|false */
    $_ = $v;
}

function test_string_arg(string $name): void {
    $v = getenv($name);
    /** @mir-check $v is string|false */
    $_ = $v;
}

function test_string_arg_with_local_only(string $name): void {
    $v = getenv($name, true);
    /** @mir-check $v is string|false */
    $_ = $v;
}

function test_literal_string_arg(): void {
    $v = getenv('FOO');
    /** @mir-check $v is string|false */
    $_ = $v;
}

function test_ambiguous_nullable_arg(?string $name): void {
    $v = getenv($name);
    /** @mir-check $v is string|array|false */
    $_ = $v;
}
===expect===
