===description===
Union, nullable, and array types validate each member.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param \int|\string $a
//        ^^^^ InvalidDocblockType: Invalid docblock type: @param backslash-qualified non-class type '\int' is not a fully qualified name
//             ^^^^^^^ InvalidDocblockType: Invalid docblock type: @param backslash-qualified non-class type '\string' is not a fully qualified name
 * @return ?\int
//          ^^^^ InvalidDocblockType: Invalid docblock type: @return backslash-qualified non-class type '\int' is not a fully qualified name
 * @var \int[] $items
//      ^^^^^^ InvalidDocblockType: Invalid docblock type: @var backslash-qualified non-class type '\int[]' is not a fully qualified name
 */
function f($a) {
    return null;
}
===expect===
