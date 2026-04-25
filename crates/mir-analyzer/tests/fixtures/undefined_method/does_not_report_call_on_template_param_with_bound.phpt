===file===
<?php
/**
 * @template T of object
 * @param T $obj
 */
function g($obj): void {
    $obj->doSomething();
}
===expect===
