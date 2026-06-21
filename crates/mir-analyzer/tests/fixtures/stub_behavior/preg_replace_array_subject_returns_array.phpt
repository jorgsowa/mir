===description===
preg_replace with an array subject returns array<int, string>, not array|null.
Null is only returned on pattern error (programming mistake), not modeled as
part of the happy-path return type.
===config===
suppress=UnusedVariable,ForbiddenCode
===file===
<?php
$inputs = ['hello', 'world'];
$result = preg_replace('/o/', '0', $inputs);
/** @mir-check $result is array<int, string> */
var_dump($result);
===expect===
