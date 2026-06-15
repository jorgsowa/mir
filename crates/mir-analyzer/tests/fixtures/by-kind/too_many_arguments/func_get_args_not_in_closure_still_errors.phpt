===description===
func_get_args() inside a closure body does not suppress TooManyArguments for the outer function
===config===
suppress=MissingClosureReturnType
===file===
<?php
function outerFn(string $x): void {
    $inner = function() {
        // func_get_args() is inside a closure — does not apply to outerFn
        return func_get_args();
    };
}

outerFn('hello', 'world');
===expect===
UnusedParam@2:17-2:26: Parameter $x is never used
UnusedVariable@3:4-3:10: Variable $inner is never read
TooManyArguments@9:17-9:24: Too many arguments for outerFn(): expected 1, got 2
