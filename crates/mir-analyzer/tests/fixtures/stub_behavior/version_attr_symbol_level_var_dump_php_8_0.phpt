===description===
Symbol-level PhpStormStubsElementAvailable: var_dump() resolves the 8.0 declaration (requires an argument)
===config===
php_version=8.0
suppress=ForbiddenCode
===file===
<?php
var_dump();
===expect===
TooFewArguments@2:1-2:11: Too few arguments for var_dump(): expected 1, got 0
