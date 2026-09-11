===description===
A backslash-qualified keyword as a `@template` bound is invalid.
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
