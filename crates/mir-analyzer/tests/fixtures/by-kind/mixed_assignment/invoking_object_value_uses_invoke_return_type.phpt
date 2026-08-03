===description===
M3: invoking an object value (`$obj(...)`) resolves __invoke()'s declared
return type instead of falling back to mixed — including the recursive
`$this(...)` self-invocation idiom.
===config===
suppress=UnusedParam
===file===
<?php
class Adder {
    public function __invoke(int $a, int $b): int { return $a + $b; }
}
function useAdder(Adder $adder): void {
    $result = $adder(1, 2);
    strlen($result);
}

class Recursive {
    public function __invoke(int $n): int {
        if ($n <= 0) {
            return 0;
        }
        return $this($n - 1);
    }
}

/**
 * A union containing the builtin `Closure` class as a bare, signature-less
 * `TNamedObject` (not a structural `TClosure`) alongside a callable(...): R
 * atom must still yield the specific return type — `Closure`'s own stub
 * `__invoke(...$_)` declares no return type, so resolving THAT one first
 * must not short-circuit the loop before it reaches the sibling atom that
 * actually knows the answer.
 * @param (callable(int): string)|Closure $gen
 */
function invokeClosureOrCallableUnion(int $n, $gen): void {
    $result = $gen($n);
    strlen($result);
}
===expect===
ArgumentTypeCoercion@7:11-7:18: Argument $string of strlen() expects 'string', got 'int' — coercion may fail at runtime
