===description===
reports too many function arguments inside wrapper
===file===
<?php
function takes_one(string $s): void {}
function wrap(): void {
    takes_one('a', 'b', 'c');
}
===expect===
UnusedParam@2:20-2:29: Parameter $s is never used
TooManyArguments@4:20-4:23: Too many arguments for takes_one(): expected 1, got 3
