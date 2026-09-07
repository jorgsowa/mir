===description===
PHP 8.3 typed class constants (`const int FOO = 1;`) skipped literal inference entirely —
type priority stopped at the native scalar hint (`int`) and never tried narrowing to the
literal value (`positive-int`), discarding precision a same-kind literal would give.
===config===
suppress=UnusedParam
===file===
<?php
final class Foo {
    private const int ID = 5;
    public function bar(): void {
        baz([self::ID]);
    }
}

/** @param list<positive-int> $ids */
function baz(array $ids): void {}

===expect===
