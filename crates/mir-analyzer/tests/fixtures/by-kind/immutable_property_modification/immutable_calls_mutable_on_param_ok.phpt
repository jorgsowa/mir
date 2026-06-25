===description===
Calling a mutable method on a parameter (not $this) inside a @psalm-immutable
class is allowed — the immutability constraint only guards $this, not other objects.
===config===
suppress=ImpurePropertyAssignment
===file===
<?php

class Accumulator {
    public int $total = 0;

    public function add(int $n): void {
        $this->total += $n;
    }
}

/** @psalm-immutable */
class Processor {
    public function __construct(public int $factor) {}

    public function process(Accumulator $acc, int $value): void {
        $acc->add($this->factor * $value);
    }
}
===expect===
