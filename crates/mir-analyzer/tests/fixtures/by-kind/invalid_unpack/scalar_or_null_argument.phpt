===description===
Unpacking a scalar or `null` into a call throws `Only arrays and Traversables can be unpacked`; arrays, iterables, objects (a subclass may be `Traversable`), `mixed` and possibly-null arrays are not reported.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function sum(int ...$numbers): int { return array_sum($numbers); }

final class Calc {
    public function __construct(int ...$numbers) {}
    public function add(int ...$numbers): void {}
    public static function mul(int ...$numbers): void {}
}

function demo(string $text, int|string $scalar, ?array $maybe, iterable $items, Countable $counted, mixed $any): void {
    sum(...1);
//      ^^^^ InvalidUnpack: Only arrays and Traversables can be unpacked, int given
    sum(...$text);
//      ^^^^^^^^ InvalidUnpack: Only arrays and Traversables can be unpacked, string given
    sum(...$scalar);
//      ^^^^^^^^^^ InvalidUnpack: Only arrays and Traversables can be unpacked, int|string given
    sum(...null);
//      ^^^^^^^ InvalidUnpack: Only arrays and Traversables can be unpacked, null given
    (new Calc())->add(...true);
//                    ^^^^^^^ InvalidUnpack: Only arrays and Traversables can be unpacked, true given
    Calc::mul(...1.5);
//            ^^^^^^ InvalidUnpack: Only arrays and Traversables can be unpacked, float given
    new Calc(...$text);
//           ^^^^^^^^ InvalidUnpack: Only arrays and Traversables can be unpacked, string given
    sum(...[1, 2]);
    sum(...$maybe);
//      ^^^^^^^^^ MixedArgument: Argument $numbers of sum() is mixed
    sum(...$items);
//      ^^^^^^^^^ MixedArgument: Argument $numbers of sum() is mixed
    sum(...$counted);
//      ^^^^^^^^^^^ MixedArgument: Argument $numbers of sum() is mixed
    sum(...$any);
//      ^^^^^^^ MixedArgument: Argument $numbers of sum() is mixed
}
