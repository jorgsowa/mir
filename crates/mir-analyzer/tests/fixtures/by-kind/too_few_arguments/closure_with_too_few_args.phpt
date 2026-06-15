===description===
Closure with too few args
===file===
<?php
/**
 * @param Closure(string, int):void $fn
 */
function test(callable $fn): void {
    $fn('hello');
}

===expect===
TooFewArguments@6:4-6:16: Too few arguments for {closure}(): expected 2, got 1
