===description===
Callable with correct args
===file===
<?php
/**
 * @param callable(string):void $fn
 */
function test(callable $fn): void {
    $fn('hello');
}

===expect===
