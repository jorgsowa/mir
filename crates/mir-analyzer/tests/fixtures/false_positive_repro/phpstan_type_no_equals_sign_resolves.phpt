===description===
M14: real PHPStan's `@phpstan-type Name Expr` syntax has no `=` (unlike
Psalm's `@psalm-type Name = Expr`) — a `@phpstan-type` alias written in that
native, no-equals form must still resolve instead of silently dropping and
leaving every reference an UndefinedDocblockClass. The `=` form must keep
working too (some codebases mix conventions), and `@psalm-type` must still
require `=` (unchanged) since real Psalm has no bare form.
===config===
suppress=UnusedParam
===file===
<?php
/** @phpstan-type NoEquals array{a: int, b: string} */
final class Foo {
    /** @return NoEquals */
    public function get(): array { return ['a' => 1, 'b' => 'x']; }
}

/** @phpstan-type WithEquals = array{a: int} */
final class Bar {
    /** @return WithEquals */
    public function get(): array { return ['a' => 1]; }
}

function needsString(string $s): void {}
needsString((new Foo())->get()['a']);
needsString((new Bar())->get()['a']);
===expect===
ArgumentTypeCoercion@15:12-15:35: Argument $s of needsString() expects 'string', got 'int' — coercion may fail at runtime
ArgumentTypeCoercion@16:12-16:35: Argument $s of needsString() expects 'string', got 'int' — coercion may fail at runtime
