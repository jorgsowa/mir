===description===
reports too many function arguments inside wrapper
===file===
<?php
function takes_one(string $s): void {}
function wrap(): void {
    takes_one('a', 'b', 'c');
}
===expect===
UnusedParam@2:19: Parameter $s is never used
TooManyArguments@4:19: Too many arguments for takes_one(): expected 1, got 3
