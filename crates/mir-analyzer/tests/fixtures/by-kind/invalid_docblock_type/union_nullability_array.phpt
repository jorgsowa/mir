===description===
Union, nullable, and array types validate each member.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param \int|\string $a
 * @return ?\int
 * @var \int[] $items
 */
function f($a) {
    return null;
}
===expect===
InvalidDocblockType@3:10-3:14: Invalid docblock type: @param backslash-qualified non-class type '\int' is not a fully qualified name
InvalidDocblockType@3:15-3:22: Invalid docblock type: @param backslash-qualified non-class type '\string' is not a fully qualified name
InvalidDocblockType@4:12-4:16: Invalid docblock type: @return backslash-qualified non-class type '\int' is not a fully qualified name
InvalidDocblockType@5:8-5:14: Invalid docblock type: @var backslash-qualified non-class type '\int[]' is not a fully qualified name
