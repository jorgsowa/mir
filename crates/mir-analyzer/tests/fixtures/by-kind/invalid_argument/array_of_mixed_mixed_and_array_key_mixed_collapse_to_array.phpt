===description===
Both `array<mixed, mixed>` and `array<array-key, mixed>` carry no more
information than the bare `array` — `array-key` (int|string) is already the
maximal legal key domain, so it's as much a "default" key as `mixed` is. Both
must display as the collapsed `array`, not the decomposed generic form.
===file===
<?php
class A {}

/** @param array<mixed, mixed> $a */
function f($a): void { $_ = $a; }

/** @param array<array-key, mixed> $b */
function g($b): void { $_ = $b; }

function test(): void {
    f(new A());
//    ^^^^^^^ InvalidArgument: Argument $a of f() expects 'array', got 'A'
    g(new A());
//    ^^^^^^^ InvalidArgument: Argument $b of g() expects 'array', got 'A'
}
===expect===
