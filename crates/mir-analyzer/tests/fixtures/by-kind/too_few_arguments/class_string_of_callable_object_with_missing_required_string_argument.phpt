===description===
Class string of callable object with missing required string argument
===file===
<?php
/**
 * @param class-string<object&callable(string):void> $className
 */
function takesCallableObject(string $className): void {
    $object = new $className();
    $object();
}
===expect===
