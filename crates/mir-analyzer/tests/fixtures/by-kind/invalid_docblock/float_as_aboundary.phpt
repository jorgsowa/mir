===description===
Float as a boundary
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param int<0, 5.5> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@3:3-3:24: Invalid docblock: @param has invalid int range boundary `5.5`: must be an integer literal, `min`, or `max`
