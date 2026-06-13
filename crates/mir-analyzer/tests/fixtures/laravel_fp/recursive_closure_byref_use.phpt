===description===
Regression (laravel/framework): a self-referential closure `$f = function () use (&$f)`
is valid — the by-ref use auto-creates the variable the assignment defines. mir now
treats a by-ref capture as defined (typed as a callable of unknown arity), so it no
longer reports UndefinedVariable / MixedFunctionCall.
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement
===file===
<?php
function build(): int {
    $factorial = function (int $n) use (&$factorial): int {
        return $n <= 1 ? 1 : $n * $factorial($n - 1);
    };
    return $factorial(5);
}
===expect===
