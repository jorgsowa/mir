===description===
in_array()/array_search() type-hint the haystack as non-nullable array —
reaching either branch of the condition proves it wasn't null, regardless
of which branch, for var/prop/static-prop receivers alike.
===config===
suppress=UnusedVariable,UnusedParam,PossiblyNullArgument
===file===
<?php

/** @param array<string, int>|null $haystack */
function test_in_array_true_branch(string $needle, ?array $haystack): void {
    if (in_array($needle, $haystack)) {
        /** @mir-check $haystack is array<string, int> */
        $_ = 1;
    }
}

/** @param array<string, int>|null $haystack */
function test_in_array_false_branch(string $needle, ?array $haystack): void {
    if (!in_array($needle, $haystack)) {
        /** @mir-check $haystack is array<string, int> */
        $_ = 1;
    }
}

/** @param array<string, int>|null $haystack */
function test_array_search_not_false(string $needle, ?array $haystack): void {
    if (array_search($needle, $haystack) !== false) {
        /** @mir-check $haystack is array<string, int> */
        $_ = 1;
    }
}

class Bag {
    /** @var array<string, int>|null */
    public ?array $items = null;
}

function test_in_array_prop_receiver(string $needle, Bag $bag): void {
    if (in_array($needle, $bag->items)) {
        /** @mir-check $bag->items is array<string, int> */
        $_ = 1;
    }
}

class StaticBag {
    /** @var array<string, int>|null */
    public static ?array $items = null;
}

function test_in_array_static_prop_receiver(string $needle): void {
    if (in_array($needle, StaticBag::$items)) {
        /** @mir-check StaticBag::$items is array<string, int> */
        $_ = 1;
    }
}

/** @param array<string, int>|null $haystack */
function test_haystack_still_nullable_outside_condition(string $needle, ?array $haystack): void {
    /** @mir-check $haystack is array<string, int>|null */
    $_ = 1;
    in_array($needle, $haystack);
}
===expect===
