===description===
Callable with too many args
===file===
<?php
/**
 * @param callable(string):void $fn
 */
function test(callable $fn): void {
    $fn('hello', 'world');
}

===expect===
TooManyArguments@6:17-6:24: Too many arguments for callable(): expected 1, got 2
