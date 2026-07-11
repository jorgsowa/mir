===description===
Regression guard for the class-template/method-template gating fix: a
`callable(T): R` parameter where T is bound to `Animal` from the receiver
still correctly rejects a closure whose own parameter only accepts the
narrower `Dog` — the callback must accept ANY `Animal` since `apply()` may
invoke it with a plain (non-Dog) Animal, so this is a real contravariance
violation, not something the class-template-corruption fix should suppress.
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
class Dog extends Animal {}

/** @param Box<Animal> $b */
function test(Box $b): void {
    $r = $b->apply(fn(Dog $d): string => "x");
}
===expect===
InvalidArgument@22:19-22:44: Argument $callback of typed_callable() expects 'callable whose parameter #1 accepts 'Animal'', got 'callable whose parameter #1 only accepts 'Dog''
