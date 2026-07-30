===description===
Negative control for the typed-class-const literal-narrowing fix: narrowing to the
literal value must not become a blanket bypass — a literal that genuinely violates the
target type (a negative int against `positive-int`) still flags.
===config===
suppress=UnusedParam
===file===
<?php
final class Foo {
    private const int ID = -5;
    public function bar(): void {
        baz([self::ID]);
    }
}

/** @param list<positive-int> $ids */
function baz(array $ids): void {}

===expect===
InvalidArgument@5:12-5:22: Argument $ids of baz() expects 'list<positive-int>', got 'array{0: -5}'
