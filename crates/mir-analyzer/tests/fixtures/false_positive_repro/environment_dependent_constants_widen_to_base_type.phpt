===description===
FP-I7: PHP_SAPI, PHP_OS, PHP_INT_SIZE, and DIRECTORY_SEPARATOR carry the
bundled stub's single define() literal (e.g. Linux/cli/'/') — every
cross-platform/SAPI guard against a DIFFERENT literal was flagged
ImpossibleIdenticalComparison.
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

function is32Bit(): bool {
    return PHP_INT_SIZE === 4;
}
===expect===
