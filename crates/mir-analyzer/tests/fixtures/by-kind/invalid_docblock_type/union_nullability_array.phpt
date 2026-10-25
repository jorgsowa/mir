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
//        ^^^^ InvalidDocblockType: @param backslash-qualified non-class type '\int' is not a fully qualified name
//             ^^^^^^^ InvalidDocblockType: @param backslash-qualified non-class type '\string' is not a fully qualified name
 * @return ?\int
//          ^^^^ InvalidDocblockType: @return backslash-qualified non-class type '\int' is not a fully qualified name
 * @var \int[] $items
//      ^^^^^^ InvalidDocblockType: @var backslash-qualified non-class type '\int[]' is not a fully qualified name
 */
function f($a) {
    return null;
}
