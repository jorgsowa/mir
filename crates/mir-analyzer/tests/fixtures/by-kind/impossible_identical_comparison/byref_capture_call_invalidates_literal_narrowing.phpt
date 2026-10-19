===description===
A call may run a closure that rewrites a by-reference capture, so literal
narrowing from `===`, `||` and `in_array` does not survive it.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function single(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($x === 'a') { $f(); if ($x === 'z') {} }
}

function orChain(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($x === 'a' || $x === 'b') { $f(); if ($x === 'z') {} }
}

function inArray(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if (in_array($x, ['a', 'b'], true)) { $f(); if ($x === 'z') {} }
}

function anyCall(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($x === 'a') { strlen('q'); if ($x === 'z') {} }
}

function methodCall(string $x, Runner $r): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($x === 'a') { $r->run($f); if ($x === 'z') {} }
}

function staticCall(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($x === 'a') { Runner::go($f); if ($x === 'z') {} }
}

function constructorCall(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($x === 'a') { new Runner($f); if ($x === 'z') {} }
}

function callInsideCondition(string $x): void {
    $f = function () use (&$x): void { $x = 'z'; };
    if ($x === 'a') { if ($f() === null && $x === 'z') {} }
}

function intLiterals(int $n): void {
    $f = function () use (&$n): void { $n = 7; };
    if ($n === 1) { $f(); if ($n === 7) {} }
}

class Runner {
    public function __construct(?callable $c = null) {}
    public function run(callable $c): void { $c(); }
    public static function go(callable $c): void { $c(); }
}
