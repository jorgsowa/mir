===description===
Throwing null fires InvalidThrow
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param null $e
 */
function throws_null($e): never {
    throw $e;
//  ^^^^^^^^^ InvalidThrow: Thrown type 'null' does not extend Throwable
}
===expect===
