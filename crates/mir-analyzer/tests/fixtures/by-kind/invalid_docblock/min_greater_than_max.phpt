===description===
Min greater than max
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param int<4, 3> $a
// ^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @param has invalid int range: min (4) must not be greater than max (3)
 */
function scope(int $a){
    return $a;
}
===expect===
