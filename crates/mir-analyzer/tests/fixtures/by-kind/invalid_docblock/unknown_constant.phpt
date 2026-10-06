===description===
Unknown constant
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param int<0, FOO> $a
// ^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @param has invalid int range boundary `FOO`: must be an integer literal, `min`, or `max`
 */
function scope(int $a){
    return $a;
}
===expect===
