===description===
preg_replace with a string subject returns string (not string|null or string[]|null).
Null is only returned on pattern error which is a programming mistake, not a type
we expose to callers.
===config===
suppress=UnusedVariable
===file===
<?php
$input = 'hello world';
$result = preg_replace('/world/', 'PHP', $input);
/** @mir-check $result is string */
echo $result;
===expect===
