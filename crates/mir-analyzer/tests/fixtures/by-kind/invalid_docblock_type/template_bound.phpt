===description===
Template bounds reject backslash-qualified keywords.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template T of \int
 * @param T $a
 */
function f($a): void {
}
===expect===
InvalidDocblockType@3:18-3:22: Invalid docblock type: @template backslash-qualified non-class type '\int' is not a fully qualified name
