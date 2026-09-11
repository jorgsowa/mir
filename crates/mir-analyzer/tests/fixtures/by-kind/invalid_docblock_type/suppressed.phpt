===description===
`@psalm-suppress InvalidDocblockType` suppresses the warning for that
docblock.
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
