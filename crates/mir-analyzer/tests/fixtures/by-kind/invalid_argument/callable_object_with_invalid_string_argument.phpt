===description===
Callable object with invalid string argument
===ignore===
TODO
===file===
<?php
/**
 * @param object&callable(string):void $object
 */
function takesCallableObject(object $object): void {
    $object(true);
}

===expect===
