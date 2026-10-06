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
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@3:3-3:24: Invalid docblock: @param has invalid int range: `max` must be the second argument, not the first
