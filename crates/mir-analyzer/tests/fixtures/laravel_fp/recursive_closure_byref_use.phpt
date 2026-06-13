===description===
Laravel FP (laravel/framework): a self-referential closure `$f = function () use (&$f)`
is valid (the by-ref use binds the variable the assignment defines), but mir reports
UndefinedVariable for $f in the use list. Ignored pending fix — see ROADMAP §1.4.
===ignore===
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
