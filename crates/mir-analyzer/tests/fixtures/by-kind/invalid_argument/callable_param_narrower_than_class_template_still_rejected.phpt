===description===
Regression guard for the class-template/method-template gating fix: a
`callable(T): R` parameter where T is bound to `Animal` from the receiver
still flags (as a coercion) a closure whose own parameter only accepts the
narrower `Dog` — `apply()` may invoke it with a plain (non-Dog) Animal, so the
contravariance gap must stay reported, not suppressed.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//                 ^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $fn of apply() expects 'callable whose parameter #1 accepts Animal', got 'callable whose parameter #1 only accepts Dog' — coercion may fail at runtime
}
