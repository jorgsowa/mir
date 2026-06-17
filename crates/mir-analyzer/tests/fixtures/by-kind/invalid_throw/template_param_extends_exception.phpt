===description===
Throwing a @template T of Exception does not fire InvalidThrow
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template T of Exception
 * @param T $e
 */
function rethrow($e): never {
    throw $e;
}
===expect===

