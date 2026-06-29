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
}
===expect===
InvalidThrow@6:4-6:13: Thrown type 'null' does not extend Throwable
