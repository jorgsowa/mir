===description===
String as a boundary
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param int<0, "bar"> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@3:3-3:26: Invalid docblock: @param has invalid int range boundary `"bar"`: must be an integer literal, `min`, or `max`
