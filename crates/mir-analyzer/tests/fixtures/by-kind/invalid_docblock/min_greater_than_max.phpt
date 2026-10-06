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
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@3:3-3:22: Invalid docblock: @param has invalid int range: min (4) must not be greater than max (3)
