===description===
Negative control: a `float`-hinted constant initialized with an int literal (PHP coerces
it to a real float value at runtime) must keep the `float` type, not narrow to the int
literal — narrowing only applies when the hint is the literal's own base scalar kind.
===config===
suppress=UnusedParam
===file===
<?php
final class Foo {
    private const float PI = 3;
    public function bar(): void {
        baz([self::PI]);
    }
}

/** @param list<positive-int> $ids */
function baz(array $ids): void {}

===expect===
InvalidArgument@5:12-5:22: Argument $ids of baz() expects 'list<positive-int>', got 'array{0: float}'
