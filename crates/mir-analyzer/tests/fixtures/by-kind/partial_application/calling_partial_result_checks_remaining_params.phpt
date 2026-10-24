===description===
The Closure a partial application creates takes the parameters its placeholders leave open.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function add(int $a, int $b): int { return $a + $b; }
function label(int $id, string $name, float $weight = 1.0): string { return $name; }
function sum(int ...$nums): int { return array_sum($nums); }

class Calc {
    public function mul(int $a, int $b): int { return $a * $b; }
    public static function div(int $a, int $b): float { return $a / $b; }

    public function own(): void {
        $double = $this->mul(?, 2);
        $double("x");
//              ^^^ InvalidArgument: Argument $a of {closure}() expects 'int', got '"x"'
        $half = self::div(?, 2);
        $half("y");
//            ^^^ InvalidArgument: Argument $a of {closure}() expects 'int', got '"y"'
    }
}

function f(Calc $calc): void {
    $addFive = add(?, 5);
    $addFive("nope");
//           ^^^^^^ InvalidArgument: Argument $a of {closure}() expects 'int', got '"nope"'
    $rest = label(1, ...);
    $rest([]);
//        ^^ InvalidArgument: Argument $name of {closure}() expects 'string', got 'array{}'
    $named = label(name: ?, id: 1);
    $named([]);
//         ^^ InvalidArgument: Argument $name of {closure}() expects 'string', got 'array{}'
    $first = sum(?, 2, 3);
    $first("z");
//         ^^^ InvalidArgument: Argument $nums of {closure}() expects 'int', got '"z"'
    $triple = $calc->mul(?, 3);
    $triple("bad");
//          ^^^^^ InvalidArgument: Argument $a of {closure}() expects 'int', got '"bad"'
    $both = add(?, ?);
    echo $both(1, 2);
    $result = $addFive(10);
    echo $result;
}

function g(?Calc $calc): void {
    $fn = fn(int $x, string $y): int => $x;
    $bound = $fn(?, "s");
    $bound("bad");
//         ^^^^^ InvalidArgument: Argument $x of {closure}() expects 'int', got '"bad"'
    $maybe = $calc?->mul(?, 2);
    $maybe("bad");
//         ^^^^^ InvalidArgument: Argument $a of {closure}() expects 'int', got '"bad"'
    $addFive = add(?, 5);
    $again = $addFive(?);
    $again("bad");
//         ^^^^^ InvalidArgument: Argument $a of {closure}() expects 'int', got '"bad"'
}
