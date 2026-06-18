===description===
LanguageLevelTypeAware return: explode() returns string[]|false (default) on PHP 7.4
===config===
php_version=7.4
suppress=UnusedVariable,PossiblyInvalidArrayAccess
===file===
<?php
$parts = explode(",", "a,b,c");
/** @mir-check $parts is non-empty-list<string>|false */
echo $parts[0];
===expect===
