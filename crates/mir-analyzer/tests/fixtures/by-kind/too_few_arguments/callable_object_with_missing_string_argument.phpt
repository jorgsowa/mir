===description===
Callable object with missing string argument
===file===
<?php
/**
 * @param object&callable(string):void $object
 */
function takesCallableObject(object $object): void {
    $object();
}

===expect===
TooFewArguments@6:4-6:13: Too few arguments for callable(): expected 1, got 0
