===description===
does not report call on generic type param
===file===
<?php
/**
 * @template T
 * @param T $obj
 */
function f($obj): void {
    $obj->method();
}
===expect===
===ignore===
TODO
