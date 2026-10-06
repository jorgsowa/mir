===description===
Min specified as second
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param int<0, min> $a
// ^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @param has invalid int range: `min` must be the first argument, not the second
 */
function scope(int $a){
    return $a;
}
===expect===
