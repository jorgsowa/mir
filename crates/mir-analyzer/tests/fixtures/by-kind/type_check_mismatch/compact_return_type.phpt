===description===
compact('a', 'b', ...) builds a precise shape from each string-literal
name's current variable type instead of a generic array. A possibly-
undefined variable's key is marked optional (compact() silently omits an
undefined name rather than including it as null). Falls back to the stub
for a non-literal/spread name.
===config===
suppress=UnusedVariable,UnusedParam,PossiblyUndefinedVariable
===file===
<?php

function test_compact_precise_shape(string $name, int $age): void {
    /** @mir-check compact('name', 'age') is array{name: string, age: int} */
    $_ = compact('name', 'age');
}

function test_compact_possibly_undefined_is_optional(bool $cond, string $name): void {
    if ($cond) {
        $renamed = $name;
    }
    /** @mir-check compact('renamed') is array{renamed?: string} */
    $_ = compact('renamed');
}

function test_compact_dynamic_name_fallback(string $key): void {
    /** @mir-check compact($key) is array */
    $_ = compact($key);
}
===expect===
