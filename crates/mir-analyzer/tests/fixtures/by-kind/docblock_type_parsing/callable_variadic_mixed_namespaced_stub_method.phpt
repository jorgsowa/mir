===description===
Bare `mixed` in a callable signature stays the builtin type for methods of a
namespaced stub class, so any typed closure parameter is accepted.
===config===
<mir>
  <stubs>
    <file name="stub.php"/>
  </stubs>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:stub.php===
<?php
namespace Lib;

final class Matcher {
    /** @param callable(mixed...):bool $callback */
    public static function postfix(callable $callback): object { return new \stdClass; }

    /** @param callable(mixed ...$args):bool $callback */
    public function named(callable $callback): void {}

    /** @param callable(...mixed):bool $callback */
    public function prefix(callable $callback): void {}

    /** @param \Closure(mixed):bool $callback */
    public function single(\Closure $callback): void {}

    /** @param callable(int, mixed...):bool $callback */
    public function mixedTail(callable $callback): void {}
}
===file:main.php===
<?php
namespace App;

use Lib\Matcher;

function t(Matcher $m): void {
    Matcher::postfix(function (array $r): bool { return true; });
    $m->named(fn (string $s, int $n): bool => $n > 0);
    $m->prefix(function (\stdClass $o): bool { return true; });
    $m->single(function (array $r): bool { return true; });
    $m->mixedTail(function (int $i, array ...$rest): bool { return true; });
    $m->mixedTail(function (string $s, array ...$rest): bool { return true; });
//                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $callback of mixedTail() expects 'callable whose parameter #1 accepts int', got 'callable whose parameter #1 only accepts string'
}

function check_postfix($x) {
    /**
     * @var callable(mixed...):bool $x
     * @mir-check $x is callable(mixed): bool
     */
    var_dump($x);
}

function check_mixed_tail($x) {
    /**
     * @var callable(int, mixed...):bool $x
     * @mir-check $x is callable(int, mixed): bool
     */
    var_dump($x);
}
===expect===
