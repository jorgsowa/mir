===description===
PhpStormStubsElementAvailable: strrchr() third param (from 8.3) absent on PHP 8.2 — extra arg is TooManyArguments
===config===
php_version=8.2
suppress=UnusedVariable
===file===
<?php
$x = strrchr("hello", "l", true);
===expect===
TooManyArguments@2:27-2:31: Too many arguments for strrchr(): expected 2, got 3
