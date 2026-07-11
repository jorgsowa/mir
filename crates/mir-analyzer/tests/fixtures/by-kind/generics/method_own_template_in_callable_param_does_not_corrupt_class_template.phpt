===description===
FP: a method's own `@template R` used inside a `callable(T): R` parameter
type (where T is the class's own template) must not let matching the
closure argument's parameter type against T rebind the class-level T —
T is already bound from the receiver and is a different template entirely
from R. `Box<Animal>::apply(callable(T): R $fn): array{0: T, 1: R}` called
with a closure accepting `Animal` (contravariantly valid for T=Animal) must
keep T as Animal, not silently rebind it to whatever the closure's own
parameter type happens to be.
===config===
suppress=MissingPropertyType,UnusedVariable
===file===
<?php
/** @template T */
class Box {
    /** @param T $item */
    public function __construct(private $item) {}

    /**
     * @template R
     * @param callable(T): R $fn
     * @return array{0: T, 1: R}
     */
    public function apply(callable $fn) {
        return [$this->item, $fn($this->item)];
    }
}

class Animal {}

/** @param Box<Animal> $b */
function test(Box $b): void {
    $r = $b->apply(fn(Animal $a): string => "x");
    /** @mir-check $r is array{0: Animal, 1: string} */
    echo 1;
}
===expect===
