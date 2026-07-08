===description===
An arrow function invoked from inside a @pure function must inherit the
pure scope, just like a regular closure — calling an impure function
through fn() => impure_fn() was previously never flagged.
===config===
suppress=UnusedVariable
===file===
<?php
function impure_fn(): void { echo "side effect"; }

/** @pure */
function test(): int {
    $f = fn() => impure_fn();
    $f();
    return 1;
}
===expect===
ImpureFunctionCall@6:17-6:28: Calling impure function impure_fn() in a @pure function
