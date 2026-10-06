===description===
Int atomics in a union absorb each other by bound containment: literals into
ranges, ranges into wider ranges or `int`; `int<0, max>` is `non-negative-int`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-negative-int $n */
function test_literal_into_named(int $n, bool $c): void {
    $r = $c ? $n : 5;
    /** @mir-check $r is non-negative-int */
    $_ = $r;
}

/** @param int<0, 10> $n */
function test_literal_into_range(int $n, bool $c): void {
    $r = $c ? $n : 5;
    /** @mir-check $r is int<0, 10> */
    $_ = $r;
}

/** @param non-negative-int $n */
function test_range_into_int(int $n, int $i, bool $c): void {
    $r = $c ? $n : $i;
    /** @mir-check $r is int */
    $_ = $r;
}

/** @param non-negative-int $n */
function takes_range(int $n): void {}

/** @param int<0, max> $n */
function test_alias(int $n): void {
    takes_range($n);
}

function test_literal_outside_range_kept(bool $c): void {
    $r = $c ? max(0, 3) : -5;
    /** @mir-check $r is int<3, 3>|-5 */
    $_ = $r;
}
