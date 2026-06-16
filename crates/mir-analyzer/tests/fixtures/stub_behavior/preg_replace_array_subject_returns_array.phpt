===description===
preg_replace with an array subject returns array|null, not string|string[]|null
===config===
suppress=UnusedVariable,ForbiddenCode
===file===
<?php
$inputs = ['hello', 'world'];
$result = preg_replace('/o/', '0', $inputs);
/** @mir-check $result is array<int, string>|null */
var_dump($result);
===expect===
