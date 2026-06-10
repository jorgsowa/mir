===description===
strtr() with 2-argument array form should not emit TooFewArguments
===file===
<?php
$result = strtr('hello world', ['hello' => 'goodbye']);
/** @mir-check $result is string */
echo $result;
===expect===
