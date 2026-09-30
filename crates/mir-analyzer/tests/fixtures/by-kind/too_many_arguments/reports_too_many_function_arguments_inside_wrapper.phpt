===description===
reports too many function arguments inside wrapper
===file===
<?php
function takes_one(string $s): void {}
//                 ^^^^^^^^^ UnusedParam: Parameter $s is never used
function wrap(): void {
    takes_one('a', 'b', 'c');
//                 ^^^ TooManyArguments: Too many arguments for takes_one(): expected 1, got 3
}
===expect===
