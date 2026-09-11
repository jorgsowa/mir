===description===
A leading backslash on a docblock type keyword (`@param \int`) is not a
fully qualified name — a backslash only qualifies class names — so the
spelling is invalid and reported as a warning. The un-backslashed keyword
is fine.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param \int $a
 * @param int $b
 */
function f($a, $b): string {
    return "x";
}
===expect===
InvalidDocblockType@3:10-3:14: Invalid docblock type: @param backslash-qualified non-class type '\int' is not a fully qualified name
