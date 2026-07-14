===description===
`list<mixed>`, `non-empty-list<mixed>`, and `non-empty-array<mixed, mixed>`
each collapse to their bare keyword form in display, same as the plain
`array<mixed, mixed>` case.
===file===
<?php
class A {}

/** @param list<mixed> $a */
function f($a): void { $_ = $a; }

/** @param non-empty-list<mixed> $b */
function g($b): void { $_ = $b; }

/** @param non-empty-array<mixed, mixed> $c */
function h($c): void { $_ = $c; }

function test(): void {
    f(new A());
    g(new A());
    h(new A());
}
===expect===
InvalidArgument@14:6-14:13: Argument $a of f() expects 'list', got 'A'
InvalidArgument@15:6-15:13: Argument $b of g() expects 'non-empty-list', got 'A'
InvalidArgument@16:6-16:13: Argument $c of h() expects 'non-empty-array', got 'A'
