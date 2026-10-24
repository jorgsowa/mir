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
        $half = self::div(?, 2);
        $half("y");
    }
}

function f(Calc $calc): void {
    $addFive = add(?, 5);
    $addFive("nope");
    $rest = label(1, ...);
    $rest([]);
    $named = label(name: ?, id: 1);
    $named([]);
    $first = sum(?, 2, 3);
    $first("z");
    $triple = $calc->mul(?, 3);
    $triple("bad");
    $both = add(?, ?);
    echo $both(1, 2);
    $result = $addFive(10);
    echo $result;
}
===expect===
InvalidArgument@12:16-12:19: Argument $a of {closure}() expects 'int', got '"x"'
InvalidArgument@14:14-14:17: Argument $a of {closure}() expects 'int', got '"y"'
InvalidArgument@20:13-20:19: Argument $a of {closure}() expects 'int', got '"nope"'
InvalidArgument@22:10-22:12: Argument $name of {closure}() expects 'string', got 'array{}'
InvalidArgument@24:11-24:13: Argument $name of {closure}() expects 'string', got 'array{}'
InvalidArgument@26:11-26:14: Argument $nums of {closure}() expects 'int', got '"z"'
InvalidArgument@28:12-28:17: Argument $a of {closure}() expects 'int', got '"bad"'
