===description===
array_filter with an `$v <op> N` predicate narrows int values to the kept range, in
either operand order; non-int atoms and unrecognized predicates leave values alone.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<positive-int> $ids */
function takesPositive(array $ids): void { echo count($ids); }

function greater(int $u): void {
    $f = array_filter([$u], fn($x) => $x > 0);
    /** @mir-check $f is array<0, int<1, max>> */
    takesPositive($f);
}

function flipped(int $u): void {
    $f = array_filter([$u], fn($x) => 1 <= $x);
    /** @mir-check $f is array<0, int<1, max>> */
    takesPositive($f);
}

function closure(int $u): void {
    $f = array_filter([$u], function ($x) { return $x >= 1; });
    /** @mir-check $f is array<0, int<1, max>> */
    takesPositive($f);
}

/** @param list<int> $l */
function generic(array $l): void {
    $f = array_filter($l, fn($x) => $x < 0);
    /** @mir-check $f is array<int, int<min, -1>> */
    echo 1;
}

function unrelatedPredicate(int $u): void {
    $f = array_filter([$u], fn($x) => $x !== 0);
    /** @mir-check $f is array<0, int> */
    echo 1;
}

function keyModeUntouched(int $u): void {
    $f = array_filter([$u], fn($k) => $k > 0, ARRAY_FILTER_USE_KEY);
    /** @mir-check $f is array<0, int> */
    echo 1;
}

function stillRejectsUnfiltered(int $u): void {
    takesPositive([$u]);
//                ^^^^ ArgumentTypeCoercion: Argument $ids of takesPositive() expects 'list<positive-int>', got 'array{0: int}' — coercion may fail at runtime
}
===expect===
