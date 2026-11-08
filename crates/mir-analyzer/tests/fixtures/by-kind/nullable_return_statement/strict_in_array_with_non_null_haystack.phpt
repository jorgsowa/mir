===description===
A strict `in_array()` match against a haystack of non-null values proves the needle isn't null.
===file===
<?php
function f(?string $fmt, string $req): string {
    $ok = explode('|', $req);
    if (in_array($fmt, $ok, true)) {
        return $fmt;
    }
    return 'json';
}
/** @param list<?string> $ok */
function g(?string $fmt, array $ok): string {
    if (in_array($fmt, $ok, true)) {
        return $fmt;
//      ^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }
    return 'json';
}
