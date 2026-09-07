===description===
FP-O (broader): negative type-guard early-exit narrows the type in the fallthrough.
`if (!is_string($x)) { throw ...; }` must narrow $x to string after the block.
`if (!is_int($n)) { return; }` must narrow $n to int after the block.
===config===
php_version=8.2
===file===
<?php

function onlyStrings(string|int $x): string {
    if (!is_string($x)) {
        throw new \InvalidArgumentException('not a string');
    }
    return $x;
}

function onlyInts(string|int $n): int {
    if (!is_int($n)) {
        return 0;
    }
    return $n;
}

function onlyPositive(int|null $n): int {
    if ($n === null) {
        throw new \InvalidArgumentException('null not allowed');
    }
    return $n;
}
===expect===
