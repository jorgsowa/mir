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
 * @param int $b
 */
function f($a, $b): string {
    return "x";
}
===expect===
InvalidDocblockType@3:10-3:14: Invalid docblock type: @param backslash-qualified non-class type '\int' is not a fully qualified name
