===description===
numeric-string can be "0" which is falsy; truthy check on numeric-string must
not mark the false branch as unreachable.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param numeric-string $s */
function test(string $s): void {
    if ($s) {
        /** @mir-check $s is numeric-string */
        $_ = $s;
    }
    // false branch must remain reachable (no RedundantCondition)
}
