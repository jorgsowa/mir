===description===
A closure inside a @pure function mutating a LOCALLY-created object (not a
captured param) is allowed — only externally-owned captures are guarded.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
class Counter {
    public int $n = 0;
    public function increment(): void {
        $this->n++;
    }
}

/** @pure */
function bump(): int {
    $c = new Counter();
    $fn = function () use ($c) {
        $c->increment();
    };
    $fn();
    return $c->n;
}
===expect===
