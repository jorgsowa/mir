===description===
Prevent string docblock type
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param string $mapper
 */
function map2(callable $mapper): void {}

map2("foo");
===expect===
