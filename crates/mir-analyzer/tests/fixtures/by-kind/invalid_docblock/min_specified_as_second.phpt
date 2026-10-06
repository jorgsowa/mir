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
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@3:3-3:24: Invalid docblock: @param has invalid int range: `min` must be the first argument, not the second
