===description===
Negative control: an explicit `@var` docblock still takes priority over both the native
hint and literal narrowing — a docblock that deliberately widens back to plain `int`
must still be honored, discarding the literal's own positive-int precision.
===config===
suppress=UnusedParam
===file===
<?php
final class Foo {
    /** @var int */
    private const int ID = 5;
    public function bar(): void {
        baz([self::ID]);
    }
}

/** @param list<positive-int> $ids */
function baz(array $ids): void {}

===expect===
InvalidArgument@6:12-6:22: Argument $ids of baz() expects 'list<positive-int>', got 'array{0: int}'
