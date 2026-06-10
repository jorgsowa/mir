===description===
Callable object with missing string argument
===ignore===
TODO
===file===
<?php
/**
 * @param object&callable(string):void $object
 */
function takesCallableObject(object $object): void {
    $object();
}

===expect===
