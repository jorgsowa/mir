===description===
Suppressing `InvalidDocblockType` hides the warning.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-suppress InvalidDocblockType
 * @param \int $a
 */
function f($a): string {
    return "x";
}
===expect===
