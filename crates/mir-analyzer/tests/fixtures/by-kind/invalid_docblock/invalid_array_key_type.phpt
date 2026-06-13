===description===
Invalid array key type
===config===
suppress=MissingParamType,UnusedParam
===file===
<?php
/**
 * @param array<float, string> $arg
 * @return void
 */
function foo($arg) {}
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @param has invalid array key type `float`: must be a subtype of int|string
