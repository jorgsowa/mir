===description===
Generic arguments reject backslash-qualified keywords case-insensitively.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param \array<int, string> $a
//        ^^^^^^^^^^^^^^^^^^^ InvalidDocblockType: Invalid docblock type: @param backslash-qualified non-class type '\array<int, string>' is not a fully qualified name
 * @return \INT
//         ^^^^ InvalidDocblockType: Invalid docblock type: @return backslash-qualified non-class type '\INT' is not a fully qualified name
 * @var \non-empty-array<int> $b
//      ^^^^^^^^^^^^^^^^^^^^^ InvalidDocblockType: Invalid docblock type: @var backslash-qualified non-class type '\non-empty-array<int>' is not a fully qualified name
 */
function f($a) {
    return 1;
}
