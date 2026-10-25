===description===
Max specified as first
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param int<max, 0> $a
// ^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: @param has invalid int range: `max` must be the second argument, not the first
 */
function scope(int $a){
    return $a;
}
