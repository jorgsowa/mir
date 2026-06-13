===description===
substr count two params 7 4 allowed
===config===
php_version=7.4
suppress=UnusedVariable
===file===
<?php
$output = 'some text with info here';
// In PHP 7.4+, substr_count accepts 2 parameters (offset/length are optional)
$count = substr_count($output, 'info');
===expect===
