===description===
PhpStormStubsElementAvailable: strrchr() third param available on PHP 8.3 — three args accepted
===config===
php_version=8.3
suppress=UnusedVariable
===file===
<?php
$x = strrchr("hello", "l", true);
===expect===
