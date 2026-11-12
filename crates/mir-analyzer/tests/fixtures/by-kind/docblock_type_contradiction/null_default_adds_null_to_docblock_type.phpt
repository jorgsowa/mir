===description===
An untyped parameter with an explicit `null` default can hold `null`, so its docblock type includes it and a `=== null` check is not a contradiction; without the default it still is. A by-reference parameter's docblock describes what is written, so it is left as is.
===file===
<?php
/** @param string $root */
function withDefault($root = NULL): string {
    /** @mir-check $root is string|null */
    if ($root === NULL) {
        $root = 'x';
    }
    return $root;
}

/** @param string $root */
function withoutDefault($root): bool {
    return $root === null;
//         ^^^^^^^^^^^^^^ DocblockTypeContradiction: Type 'string' makes '$root === null' impossible — this can never hold
}

/** @param int $count */
function out(&$count = null): void {
    $count = 1;
}

function callsOut(): int {
    out($n);
    return $n;
}
