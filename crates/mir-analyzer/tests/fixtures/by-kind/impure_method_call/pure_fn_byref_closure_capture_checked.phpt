===description===
The param_names extension that lets a purity check see through a captured
parameter (`pure_fn_closure_inherits_purity_context.phpt`) only ever
looked at BY-VALUE `use` entries — a by-ref capture (`use (&$c)`) is the
SAME external variable, not a copy, so it's just as externally observable,
but was silently excluded from the extension entirely, letting a mutating
method call or a property write through it bypass @pure completely.
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
function bumpViaMethod(Counter $c): void {
    $fn = function () use (&$c) {
        $c->increment();
    };
    $fn();
}

/** @pure */
function bumpViaProperty(Counter $c): void {
    $fn = function () use (&$c) {
        $c->n = 5;
    };
    $fn();
}
===expect===
ImpureMethodCall@12:8-12:23: Calling impure method increment() in a pure or immutable context
ImpurePropertyAssignment@20:8-20:17: Assigning to property n of a parameter in a pure or external-mutation-free context
