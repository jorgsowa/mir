===description===
preg_replace with a string subject returns string|null, not string|string[]|null.
When subject is a known string, the return type is string|null.
===config===
suppress=UnusedVariable
===file===
<?php
$input = 'hello world';
$result = preg_replace('/world/', 'PHP', $input);
/** @mir-check $result is string|null */
echo $result ?? '';
===expect===
