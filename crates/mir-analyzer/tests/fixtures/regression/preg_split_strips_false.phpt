===description===
FP-C: preg_split returns list<string>|false in stubs but false only fires on
a bad regex. Strip the false case so callers returning list<string> don't get
NullableReturnStatement or type errors.
===config===
php_version=8.2
===file===
<?php

/**
 * @return list<string>
 */
function split_words(string $s): array {
    return preg_split('/\s+/', $s);
}

function count_parts(string $s): int {
    $parts = preg_split('/,/', $s);
    return count($parts);
}
===expect===
