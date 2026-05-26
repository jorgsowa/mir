===description===
conditional return type resolves to if-true branch when null is passed
===config===
suppress=UnusedParam
===file===
<?php

/**
 * @template T of object
 */
class Box {}

/**
 * @template T of object
 * @param class-string<T>|null $cls
 * @return ($cls is null ? Box<object> : Box<T>)
 */
function make(?string $cls = null): mixed { throw new \RuntimeException(); }

$b = make(null);
/** @mir-check $b is Box<object> */
echo "ok";
===expect===
