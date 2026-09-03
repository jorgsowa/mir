===description===
FALSE POSITIVE reproducer. Valid PHP: passing a literal int to a `string`-typed param is a benign
coercion in non-strict mode. Example from Laravel ManagesFrequencies::spliceIntoPosition(1, 0).
mir previously emitted InvalidArgument (Error); should be ArgumentTypeCoercion (Info).
===config===
suppress=UnusedParam
php_version=8.4
===file===
<?php
/**
 * @param string $position
 * @param string $value
 */
function spliceIntoPosition(string $position, string $value): void {}

spliceIntoPosition(1, 0);
===expect===
ArgumentTypeCoercion@8:19-8:20: Argument $position of spliceIntoPosition() expects 'string', got '1' — coercion may fail at runtime
ArgumentTypeCoercion@8:22-8:23: Argument $value of spliceIntoPosition() expects 'string', got '0' — coercion may fail at runtime
