===description===
`@param array<string,int> ...$maps` binds each variadic argument to a
whole `array<string,int>` map (`$maps` is `list<array<string,int>>`) —
a string-only key means the docblock describes the PER-ARGUMENT type,
not the aggregate, unlike an int-keyed `array<int,V>`/`list<V>` which IS
already the aggregate.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string,int> ...$maps */
function stringKeyed(...$maps): void {
    foreach ($maps as $m) {
        /** @mir-check $m is array<string, int> */
        $_ = $m;
    }
}

// Negative: an int-keyed (or unspecified-key) array/list docblock already
// directly describes $sets/$lists's own aggregate shape — the existing,
// unaffected behavior this fix must not disturb.
/** @param array<int,int> ...$sets */
function intKeyed(...$sets): void {
    /** @mir-check $sets is array<int, int> */
    $_ = $sets;
}

/** @param list<int> ...$lists */
function listKeyed(...$lists): void {
    /** @mir-check $lists is list<int> */
    $_ = $lists;
}
===expect===
