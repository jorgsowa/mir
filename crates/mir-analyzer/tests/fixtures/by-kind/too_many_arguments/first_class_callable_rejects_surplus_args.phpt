===description===
A first-class callable of a named function or method reports surplus arguments
like a direct call; a closure literal accepts them (`func_get_args()`).
===file===
<?php
final class Calc {
    public function add(int $a, int $b): int { return $a + $b; }
    public static function neg(int $a): int { return -$a; }
}

function two(int $a, int $b): int { return $a + $b; }

function f(Calc $c): void {
    $len = strlen(...);
    $len('a', 'b');
//            ^^^ TooManyArguments: Too many arguments for {closure}(): expected 1, got 2
    strlen(...)('a', 'b');
//                   ^^^ TooManyArguments: Too many arguments for {closure}(): expected 1, got 2
    $two = two(...);
    $two(1, 2, 3);
//             ^ TooManyArguments: Too many arguments for {closure}(): expected 2, got 3
    $add = $c->add(...);
    $add(1, 2, 3);
//             ^ TooManyArguments: Too many arguments for {closure}(): expected 2, got 3
    $neg = Calc::neg(...);
    $neg(1, 2);
//          ^ TooManyArguments: Too many arguments for {closure}(): expected 1, got 2
    $lit = function (int $a): int { return $a; };
    $lit(1, 2);
    $arrow = fn (int $a): int => $a;
    $arrow(1, 2);
    $ok = sprintf(...);
    $ok('%s %s', 'a', 'b');
}
