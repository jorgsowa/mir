===description===
Closure::bind chained with invoke returns correct type (key regression)
===config===
suppress=UnusedVariable
===file===
<?php
class A {
    public function getData(): int { return 123; }
}

class B {}

// Original closure returns int
$closure = function(): int { return 456; };
/** @mir-check $closure is Closure(): int */

// Bind to B instance
$bound = Closure::bind($closure, new B());
/** @mir-check $bound is Closure(): int|null */

// If bound succeeds, calling it should return int, not mixed
if ($bound !== null) {
    $result = $bound();
    // This is the KEY regression test: without the fix, $bound() would return mixed
    // because $bound is ?Closure (unparam) and $bound() would extract no type info
    /** @mir-check $result is int */
}
===expect===
