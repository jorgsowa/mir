===description===
Callable object with missing string argument
===file===
<?php
/**
 * @param object&callable(string):void $object
 */
function takesCallableObject(object $object): void {
    $object();
//  ^^^^^^^^^ TooFewArguments: Too few arguments for callable(): expected 1, got 0
}

===expect===
