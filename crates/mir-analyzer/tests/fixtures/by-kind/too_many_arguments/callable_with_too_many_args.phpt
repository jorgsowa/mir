===description===
Callable with too many args
===ignore===
TODO
===file===
<?php
/**
 * @param callable(string):void $fn
 */
function test(callable $fn): void {
    $fn('hello', 'world');
}

===expect===
