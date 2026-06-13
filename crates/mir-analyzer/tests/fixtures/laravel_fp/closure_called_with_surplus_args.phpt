===description===
Regression (laravel/framework): PHP silently ignores surplus positional arguments
passed to a closure (they remain available via func_get_args). mir no longer emits
TooManyArguments for a direct closure call with extra args (named functions and
methods still keep the lint).
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
