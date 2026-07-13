===description===
A @param array shape wrapped across multiple lines is still parsed and checked,
not silently dropped to an unchecked parameter.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param array{
 *     id: int,
 *     name: string,
 * } $data
 */
function f(array $data): void {}

f("not an array");
===expect===
InvalidArgument@10:2-10:16: Argument $data of f() expects 'array{'id': int, 'name': string}', got '"not an array"'
