===description===
Generic arguments reject backslash-qualified keywords case-insensitively.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param \array<int, string> $a
 * @return \INT
 * @var \non-empty-array<int> $b
 */
function f($a) {
    return 1;
}
===expect===
InvalidDocblockType@3:10-3:29: Invalid docblock type: @param backslash-qualified non-class type '\array<int, string>' is not a fully qualified name
InvalidDocblockType@4:11-4:15: Invalid docblock type: @return backslash-qualified non-class type '\INT' is not a fully qualified name
InvalidDocblockType@5:8-5:29: Invalid docblock type: @var backslash-qualified non-class type '\non-empty-array<int>' is not a fully qualified name
