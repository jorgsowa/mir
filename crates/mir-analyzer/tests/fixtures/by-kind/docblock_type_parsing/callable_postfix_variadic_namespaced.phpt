===description===
Postfix variadic params in `callable`/`Closure` signatures keep their builtin
type (`mixed...` is never `Ns\mixed`) inside a namespace, so narrower closures
are accepted.
===config===
suppress=MissingReturnType,MissingParamType,ForbiddenCode
===file===
<?php
namespace App;

/** @param callable(mixed...):bool $f */
function run(callable $f): void { $f(1); }

run(function (int $x): bool { return true; });
run(fn (string $s, int $n): bool => $n > 0);

/** @param Closure(int...):void $f */
function each_int(\Closure $f): void { $f(1, 2); }

each_int(function (int $x): void {});
each_int(function (string $x): void {});
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $f of each_int() expects 'callable whose parameter #1 accepts int', got 'callable whose parameter #1 only accepts string'

function check_mixed_variadic($x) {
    /**
     * @var callable(mixed...):bool $x
     * @mir-check $x is callable(mixed): bool
     */
    var_dump($x);
}

function check_spaced_variadic($x) {
    /**
     * @var callable(int ...$rest):void $x
     * @mir-check $x is callable(int): void
     */
    var_dump($x);
}

function check_prefix_variadic($x) {
    /**
     * @var Closure(...string):void $x
     * @mir-check $x is Closure(string): void
     */
    var_dump($x);
}

function check_non_variadic($x) {
    /**
     * @var callable(mixed, int):bool $x
     * @mir-check $x is callable(mixed, int): bool
     */
    var_dump($x);
}
===expect===
