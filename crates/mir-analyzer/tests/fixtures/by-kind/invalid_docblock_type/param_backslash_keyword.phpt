===description===
Backslash-qualified parameter keywords are invalid; unqualified keywords are valid.
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
 * @param int $b
 */
function f($a, $b): string {
    return "x";
}
===expect===
