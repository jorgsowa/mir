===description===
FP-C: grapheme_strlen returns int|false|null in stubs, but false/null only on
invalid UTF-8 input. Normal usage should not emit NullableReturnStatement.
===config===
suppress=UnusedVariable,UnusedParam
php_version=8.2
===file===
<?php

function measure(string $s): int {
    return grapheme_strlen($s);
}

function measure_with_check(string $s): int {
    $len = grapheme_strlen($s);
    /** @mir-check $len is int */
    return $len;
}
===expect===
