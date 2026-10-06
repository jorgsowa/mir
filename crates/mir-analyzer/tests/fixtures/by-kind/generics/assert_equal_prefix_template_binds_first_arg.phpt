===description===
`@psalm-assert =T $actual` and `@phpstan-assert =T $actual` bind T from the argument bound to `$expected`, across static, instance, named-arg and variable-argument calls.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Vendor;

class Guard {
    /**
     * @template Expected
     * @param Expected $expected
     * @param mixed $actual
     * @psalm-assert =Expected $actual
     */
    public static function same(mixed $expected, mixed $actual): void {}

    /**
     * @template Expected
     * @param Expected $expected
     * @param mixed $actual
     * @phpstan-assert =Expected $actual
     */
    public function sameInstance(mixed $expected, mixed $actual): void {}
}

class Box {}

function takesInt(int $i): void {}

function staticLiteral(mixed $v): void {
    Guard::same(5, $v);
    takesInt($v);
    /** @mir-check $v is 5 */
    echo "ok";
}

function staticString(mixed $v): void {
    Guard::same('a', $v);
    /** @mir-check $v is 'a' */
    echo "ok";
}

function staticNamedArgs(mixed $v): void {
    Guard::same(actual: $v, expected: 5);
    /** @mir-check $v is 5 */
    echo "ok";
}

function staticVariableArg(int $expected, mixed $v): void {
    Guard::same($expected, $v);
    /** @mir-check $v is int */
    echo "ok";
}

function staticObjectArg(Box $expected, mixed $v): void {
    Guard::same($expected, $v);
    /** @mir-check $v is Vendor\Box */
    echo "ok";
}

function instanceCall(Guard $g, mixed $v): void {
    $g->sameInstance(5, $v);
    takesInt($v);
    /** @mir-check $v is 5 */
    echo "ok";
}
