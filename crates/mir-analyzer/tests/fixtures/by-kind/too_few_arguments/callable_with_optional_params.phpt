===description===
Callable with optional params
===file===
<?php
/**
 * @param callable(string, int=):void $fn
 */
function test(callable $fn): void {
    $fn('hello');
}

===expect===
