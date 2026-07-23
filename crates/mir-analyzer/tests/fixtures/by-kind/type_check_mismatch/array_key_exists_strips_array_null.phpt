===description===
array_key_exists()/key_exists() throw a TypeError on a null 2nd arg, so
reaching either branch proves the array argument itself wasn't null, for
var/prop/static-prop receivers alike.
===config===
suppress=UnusedVariable,UnusedParam,PossiblyNullArgument
===file===
<?php

/** @param array{a: int}|null $arr */
function test_var_true_branch(?array $arr): void {
    if (array_key_exists('a', $arr)) {
        /** @mir-check $arr is array{a: int} */
        $_ = 1;
    }
}

/** @param array{a?: int}|null $arr */
function test_var_false_branch(?array $arr): void {
    if (!array_key_exists('a', $arr)) {
        /** @mir-check $arr is array{a?: int} */
        $_ = 1;
    }
}

class Bag {
    /** @var array{a: int}|null */
    public ?array $data = null;
}

function test_prop_true_branch(Bag $bag): void {
    if (array_key_exists('a', $bag->data)) {
        /** @mir-check $bag->data is array{a: int} */
        $_ = 1;
    }
}

function test_prop_false_branch(Bag $bag): void {
    if (!array_key_exists('a', $bag->data)) {
        // 'a' is declared required (non-optional) on a lone, non-union
        // shape — treated as a hint, not proof, so the shape itself is
        // left unchanged; only null is stripped (reachability-proven).
        /** @mir-check $bag->data is array{a: int} */
        $_ = 1;
    }
}

class StaticBag {
    /** @var array{a: int}|null */
    public static ?array $data = null;
}

function test_static_prop_true_branch(): void {
    if (array_key_exists('a', StaticBag::$data)) {
        /** @mir-check StaticBag::$data is array{a: int} */
        $_ = 1;
    }
}

/** @param array{a: int}|null $arr */
function test_still_nullable_outside_condition(?array $arr): void {
    /** @mir-check $arr is array{a: int}|null */
    $_ = 1;
    array_key_exists('a', $arr);
}
===expect===
