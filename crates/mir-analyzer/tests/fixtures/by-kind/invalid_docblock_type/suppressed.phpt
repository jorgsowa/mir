===description===
Suppressing `InvalidDocblockType` hides the warning.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @psalm-suppress InvalidDocblockType
 * @param \int $a
 */
function f($a): string {
    return "x";
}
