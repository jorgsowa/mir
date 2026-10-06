===description===
Backslash-qualified docblock keywords are not undefined classes and are invalid types.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param \int $a
//        ^^^^ InvalidDocblockType: Invalid docblock type: @param backslash-qualified non-class type '\int' is not a fully qualified name
 * @param \boolean $b
//        ^^^^^^^^ InvalidDocblockType: Invalid docblock type: @param backslash-qualified non-class type '\boolean' is not a fully qualified name
 * @return \interface-string
//         ^^^^^^^^^^^^^^^^^ InvalidDocblockType: Invalid docblock type: @return backslash-qualified non-class type '\interface-string' is not a fully qualified name
 */
function f($a, $b): string {
    return "x";
}

/** @return \int-mask<1, 2, 4> */
//          ^^^^^^^^^^^^^^^^^^ InvalidDocblockType: Invalid docblock type: @return backslash-qualified non-class type '\int-mask<1, 2, 4>' is not a fully qualified name
function flags() {
    return 3;
}
