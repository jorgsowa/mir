===description===
Fully-qualified (leading `\`) get_class()/gettype()/get_debug_type() calls
narrow exactly like the bare name.
===config===
suppress=UnusedVariable,UnusedParam,MixedArgument
===file===
<?php
class Foo {}

function test_get_class(object $x): void {
    if (\get_class($x) === Foo::class) {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}

/** @param int|string $x */
function test_gettype(mixed $x): void {
    if (\gettype($x) === 'string') {
        /** @mir-check $x is string */
        $_ = $x;
    }
}

/** @param int|string $x */
function test_gettype_not_string(mixed $x): void {
    if (\gettype($x) !== 'string') {
        /** @mir-check $x is int */
        $_ = $x;
    }
}

/** @param int|string $x */
function test_get_debug_type(mixed $x): void {
    if (\get_debug_type($x) === 'string') {
        /** @mir-check $x is string */
        $_ = $x;
    }
}
===expect===
