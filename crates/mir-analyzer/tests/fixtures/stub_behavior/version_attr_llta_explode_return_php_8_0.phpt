===description===
LanguageLevelTypeAware return: explode() returns string[] (no false) on PHP 8.0
===config===
php_version=8.0
suppress=UnusedVariable
===file===
<?php
$parts = explode(",", "a,b,c");
/** @mir-check $parts is array<int, string> */
echo $parts[0];
===expect===
