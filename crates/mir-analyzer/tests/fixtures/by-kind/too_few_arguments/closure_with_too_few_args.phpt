===description===
Closure with too few args
===file===
<?php
/**
 * @param Closure(string, int):void $fn
 */
function test(callable $fn): void {
    $fn('hello');
//  ^^^^^^^^^^^^ TooFewArguments: Too few arguments for {closure}(): expected 2, got 1
}
