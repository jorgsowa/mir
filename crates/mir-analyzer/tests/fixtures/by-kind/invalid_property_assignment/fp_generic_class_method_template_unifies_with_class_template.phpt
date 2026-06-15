===description===
FP: inside a generic class, a property typed with the class template (A) must accept a value
typed with a method-level template (R) — R should unify with A, not trigger InvalidPropertyAssignment
===file===
<?php

/**
 * @template L
 * @template R
 */
class Box {}

/**
 * @template T
 * @param T $value
 * @return Box<never, T>
 */
function wrap(mixed $value): Box {
    return new Box();
}

/**
 * @template A
 */
class Container {
    /** @var Box<never, A>|null */
    private ?Box $slot = null;

    /**
     * @template R
     * @param R $item
     */
    public function store(mixed $item): void {
        $this->slot = wrap($item);
    }
}
===expect===
UnusedParam@14:14-14:26: Parameter $value is never used
