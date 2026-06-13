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
UnusedParam@2:18-2:27: Parameter $x is never used
UnusedVariable@3:5-3:11: Variable $inner is never read
TooManyArguments@9:18-9:25: Too many arguments for outerFn(): expected 1, got 2
