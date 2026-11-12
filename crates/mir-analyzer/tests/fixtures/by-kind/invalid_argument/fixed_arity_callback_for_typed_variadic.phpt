===description===
A typed variadic callable may be called with no arguments there, so a callback requiring one is rejected; `mixed...` accepts any callback.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

namespace App;

/** @param callable(float ...$values): (int|null) $callback */
function takesVariadic(callable $callback): void {}

/** @param callable(string, int ...$rest): void $callback */
function takesPrefixedVariadic(callable $callback): void {}

/** @param callable(mixed ...$args): mixed $callback */
function takesAny(callable $callback): void {}

takesVariadic(static fn (float ...$values): int => count($values));
takesVariadic(static fn (float $value = 0.0): int => 1);
takesVariadic(static fn (float $value): int => 1);
//            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $callback of takesVariadic() expects 'callable accepting variadic parameter #1', got 'callable requiring parameter #1'
takesPrefixedVariadic(static function (string $name): void {});
takesPrefixedVariadic(static function (string $name, int ...$rest): void {});
takesAny(static fn (int $value): int => $value);
