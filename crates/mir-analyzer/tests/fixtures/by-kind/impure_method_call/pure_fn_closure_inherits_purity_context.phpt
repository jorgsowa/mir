===description===
FN: a closure invoked inside a @pure function did not inherit is_in_pure_fn,
and by-value-captured objects were never added to param_names, so mutating
a captured object through an immediately-invoked closure went unflagged —
smuggling an observable side effect out of a function claimed to be pure.
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
function bump(Counter $c): void {
    $fn = function () use ($c) {
        $c->increment();
    };
    $fn();
}
===expect===
ImpureMethodCall@12:8-12:23: Calling impure method increment() in a pure or immutable context
