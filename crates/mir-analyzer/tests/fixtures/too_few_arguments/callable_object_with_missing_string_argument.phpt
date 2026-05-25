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
TooFewArguments
===ignore===
TODO
