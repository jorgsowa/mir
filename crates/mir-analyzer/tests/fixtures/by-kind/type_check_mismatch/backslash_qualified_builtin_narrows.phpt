===description===
Fully-qualified (leading `\`) builtin calls narrow exactly like the bare
name: is_*() predicates, array_is_list(), assert(), method_exists(),
property_exists().
===config===
suppress=UnusedVariable,UnusedParam,PossiblyInvalidArgument,MixedArgument,MixedMethodCall
===file===
<?php
class Foo {
    public int $bar = 1;
    public function baz(): void {}
}

/** @param int|string $x */
function test_is_string(mixed $x): void {
    if (\is_string($x)) {
        /** @mir-check $x is string */
        $_ = $x;
    } else {
        /** @mir-check $x is int */
        $_ = $x;
    }
}

/** @param int|string $x */
function test_is_int(mixed $x): void {
    if (\is_int($x)) {
        /** @mir-check $x is int */
        $_ = $x;
    }
}

/** @param array<int, int>|string $x */
function test_is_array(mixed $x): void {
    if (\is_array($x)) {
        /** @mir-check $x is array<int, int> */
        $_ = $x;
    }
}

/** @param list<int>|array<string, int> $x */
function test_array_is_list(mixed $x): void {
    if (\array_is_list($x)) {
        /** @mir-check $x is list<int> */
        $_ = $x;
    }
}

function test_assert(mixed $x): void {
    \assert($x instanceof Foo);
    /** @mir-check $x is Foo */
    $_ = $x;
}

function test_method_exists(mixed $x): void {
    if (\method_exists($x, 'baz')) {
        /** @mir-check $x is object */
        $_ = $x;
    }
}

function test_property_exists(mixed $x): void {
    if (\property_exists($x, 'bar')) {
        /** @mir-check $x is object */
        $_ = $x;
    }
}
===expect===
