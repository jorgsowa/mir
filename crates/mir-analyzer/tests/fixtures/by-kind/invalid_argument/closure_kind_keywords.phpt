===description===
`pure-closure`, `static-closure` and `static-pure-closure` are closures, not any `Closure` object: a callable string is rejected, a closure with visible side effects is rejected where purity is required, and a non-static closure literal is rejected where a static one is required.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @param pure-closure $value */
function pure($value): void {}

/** @param static-closure $value */
function staticOnly($value): void {}

/** @param static-pure-closure $value */
function staticPure($value): void {}

/** @return static-pure-closure */
function makesClosure() {
    return static fn (int $value): int => $value + 1;
}

function takesClosure(Closure $closure): void {}

takesClosure(makesClosure());

$impure = static function (): void {
    echo 'side effect';
};

pure(static fn (int $value): int => $value + 1);
pure('strlen');
//   ^^^^^^^^ InvalidArgument: Argument $value of pure() expects 'pure-Closure(mixed): mixed', got '"strlen"'
pure($impure);
//   ^^^^^^^ InvalidArgument: Argument $value of pure() expects 'pure-Closure(mixed): mixed', got 'impure Closure(): void'

staticOnly(static fn (): int => 1);
staticOnly(fn (): int => 1);
//         ^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of staticOnly() expects 'Closure(mixed): mixed', got 'non-static Closure(): positive-int'

staticPure(static fn (int $value): int => $value + 1);
staticPure(fn (int $value): int => $value + 1);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of staticPure() expects 'pure-Closure(mixed): mixed', got 'non-static Closure(int): int'
staticPure($impure);
//         ^^^^^^^ InvalidArgument: Argument $value of staticPure() expects 'pure-Closure(mixed): mixed', got 'impure Closure(): void'
