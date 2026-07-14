===description===
A bare (unparameterized) `Traversable` paired with a *more specific* array
(e.g. `array<int, string>`) must NOT collapse to `iterable<int, string>` —
the bare `Traversable` side carries no key/value guarantee, so collapsing
would overclaim precision. The decomposed union prints as-is.
===file===
<?php
class A {}

/** @param array<int, string>|Traversable $x */
function f($x): void { $_ = $x; }

function test(): void {
    f(new A());
}
===expect===
InvalidArgument@8:6-8:13: Argument $x of f() expects 'array<int, string>|Traversable', got 'A'
