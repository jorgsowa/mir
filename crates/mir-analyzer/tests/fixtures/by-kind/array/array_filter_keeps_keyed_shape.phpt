===description===
array_filter on a keyed shape keeps the keys, each made optional; falsy dropped; a
predicate callback still narrows values, key/both modes leave them alone,
open shapes stay open, and list shapes keep the generic result.
===config===
suppress=UnusedVariable
===file===
<?php
/** @param array{code: int|null, key: string|null} $s */
function plain(array $s): void {
    $f = array_filter($s);
    /** @mir-check $f is array{'code'?: int, 'key'?: non-empty-string} */
    echo 1;
}

/** @param array{code: int|null, key: string|null} $s */
function predicate(array $s): void {
    $f = array_filter($s, fn($v) => $v !== null);
    /** @mir-check $f is array{'code'?: int, 'key'?: string} */
    echo 1;
}

/** @param array{code: int|null} $s */
function byKey(array $s): void {
    $f = array_filter($s, fn($k) => $k === 'code', ARRAY_FILTER_USE_KEY);
    /** @mir-check $f is array{'code'?: int|null} */
    echo 1;
}

/** @param array{code: int|null, ...} $s */
function open(array $s): void {
    $f = array_filter($s);
    /** @mir-check $f is array{'code'?: int} */
    echo 1;
}

/** @param array{code?: int, key?: string} $e */
function wantsShape(array $e): void { echo count($e); }

function pass(): void {
    wantsShape(array_filter(['code' => 1, 'key' => 'k']));
}

/** @param list<int> $l */
function lst(array $l): void {
    $f = array_filter($l);
    /** @mir-check $f is array<int, int> */
    echo 1;
}
===expect===
