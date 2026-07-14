===description===
gettype($x) === 'literal' and get_debug_type($x) === 'literal'/Foo::class
narrow $x the same way is_string()/is_int()/etc and get_class() do.
===config===
suppress=UnusedVariable,UnusedParam,MixedArgument
===file===
<?php
class Foo {}

/** @param int|string $x */
function test_gettype_string(mixed $x): void {
    if (gettype($x) === 'string') {
        /** @mir-check $x is string */
        $_ = $x;
    }
}

/** @param int|string $x */
function test_gettype_integer_reversed(mixed $x): void {
    if ('integer' === gettype($x)) {
        /** @mir-check $x is int */
        $_ = $x;
    }
}

/** @param int|string $x */
function test_gettype_not_string(mixed $x): void {
    if (gettype($x) !== 'string') {
        /** @mir-check $x is int */
        $_ = $x;
    }
}

/** @param int|string $x */
function test_get_debug_type_string(mixed $x): void {
    if (get_debug_type($x) === 'string') {
        /** @mir-check $x is string */
        $_ = $x;
    }
}

/** @param Foo|string $x */
function test_get_debug_type_class_literal(mixed $x): void {
    if (get_debug_type($x) === 'Foo') {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}

/** @param Foo|string $x */
function test_get_debug_type_class_const(mixed $x): void {
    if (get_debug_type($x) === Foo::class) {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}

/** @param Foo|string $x */
function test_get_debug_type_class_const_reversed(mixed $x): void {
    if (Foo::class === get_debug_type($x)) {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}
===expect===
