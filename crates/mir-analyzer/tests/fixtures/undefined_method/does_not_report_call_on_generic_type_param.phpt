===source===
<?php
/**
 * @template T
 * @param T $obj
 */
function f($obj): void {
    $obj->method();
}
===expect===
