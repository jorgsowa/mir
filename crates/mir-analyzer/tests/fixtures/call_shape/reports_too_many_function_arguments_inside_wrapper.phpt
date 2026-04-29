===file===
<?php
function takes_one(string $s): void {}
function wrap(): void {
    takes_one('a', 'b', 'c');
}
===expect===
UnusedParam: Parameter $s is never used
TooManyArguments: Too many arguments for takes_one(): expected 1, got 3
