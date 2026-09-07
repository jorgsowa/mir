===description===
FP-I7: PHP_SAPI, PHP_OS, PHP_OS_FAMILY, PHP_INT_SIZE, and DIRECTORY_SEPARATOR
carry the bundled stub's single define() literal (e.g. Linux/cli/'/') — every
cross-platform/SAPI guard against a DIFFERENT literal was flagged
ImpossibleIdenticalComparison. PHP_OS_FAMILY reproduced independently in both
phpunit-phpunit's Util/ExcludeList.php and guzzlehttp/guzzle's
Handler/ProxyEnvironment.php via harness/.
===config===
suppress=UnusedVariable
===file===
<?php

function isWindows(): bool {
    return '\\' === DIRECTORY_SEPARATOR;
}

function isCgi(): bool {
    return PHP_SAPI === 'cgi-fcgi';
}

function isDarwin(): bool {
    return PHP_OS === 'Darwin';
}

function isWindowsFamily(): bool {
    return PHP_OS_FAMILY === 'Windows';
}

function is32Bit(): bool {
    return PHP_INT_SIZE === 4;
}
===expect===
