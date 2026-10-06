===description===
A by-ref capture that a closure appends to becomes a list of the written value type, since the closure may run any number of times.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Emitter {
    public function on(callable $f): void { $f(1); }
}

function appendParam(Emitter $e): void {
    $items = [];
    $e->on(function (int $n) use (&$items) { $items[] = $n; });
    /** @mir-check $items is list<int> */
    $_ = $items;
    echo $items[1];
}

function appendTwice(Emitter $e): void {
    $items = [];
    $e->on(function (int $n) use (&$items) {
        $items[] = $n;
        $items[] = 'x';
    });
    /** @mir-check $items is list<int|"x"> */
    $_ = $items;
    echo $items[3];
}

function appendToSeededList(Emitter $e): void {
    $items = ['a'];
    $e->on(function (int $n) use (&$items) { $items[] = $n; });
    echo $items[2];
}

function appendInBranch(Emitter $e): void {
    $items = [];
    $e->on(function (int $n) use (&$items) {
        if ($n > 0) {
            $items[] = $n;
        }
    });
    echo $items[1];
}

function appendInNestedClosure(Emitter $e): void {
    $items = [];
    $e->on(function () use (&$items, $e) {
        $e->on(function (int $n) use (&$items) { $items[] = $n; });
    });
    echo $items[1];
}

function keyedWriteStaysShape(Emitter $e): void {
    $items = [];
    $e->on(function (int $n) use (&$items) { $items['a'] = $n; });
    echo $items['b'];
//              ^^^ NonExistentArrayOffset: Array offset 'b' does not exist
}

function byValueStaysEmpty(Emitter $e): void {
    $items = [];
    $e->on(function (int $n) use ($items) { $items[] = $n; });
    echo $items[0];
//              ^ NonExistentArrayOffset: Array offset '0' does not exist
}

function noWriteStaysEmpty(Emitter $e): void {
    $items = [];
    $e->on(function (int $n) use (&$items) { return count($items); });
    echo $items[0];
//              ^ NonExistentArrayOffset: Array offset '0' does not exist
}
