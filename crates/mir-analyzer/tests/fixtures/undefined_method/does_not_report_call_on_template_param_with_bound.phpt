===description===
does not report call on template param with bound
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
===ignore===
TODO
