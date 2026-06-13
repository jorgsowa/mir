===description===
Laravel FP (laravel/framework): PHP silently ignores surplus positional arguments
passed to a closure/function (they remain available via func_get_args), but mir
checks the call against the closure's declared arity and emits TooManyArguments.
Ignored pending fix — see ROADMAP §1.4.
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement
===file===
<?php
function dispatchPair(): void {
    $callback = function (int $a): int {
        return $a;
    };
    $callback(1, 2);
}
===expect===
