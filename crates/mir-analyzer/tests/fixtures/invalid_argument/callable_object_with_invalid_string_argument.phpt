===description===
Callable object with invalid string argument
===file===
<?php
/**
 * @param object&callable(string):void $object
 */
function takesCallableObject(object $object): void {
    $object(true);
}

===expect===
InvalidArgument
===ignore===
TODO
