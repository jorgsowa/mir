===description===
`unset()` of an element may leave a non-empty array empty.
===file===
<?php
/** @param non-empty-array<string, int> $a */
function f(array $a): void {
    unset($a['static']);
    /** @mir-check $a is array<string, int> */
    echo 1;
}

/** @param non-empty-list<int> $l */
function g(array $l, int $i): void {
    unset($l[$i]);
    /** @mir-check $l is list<int> */
    echo 1;
}
