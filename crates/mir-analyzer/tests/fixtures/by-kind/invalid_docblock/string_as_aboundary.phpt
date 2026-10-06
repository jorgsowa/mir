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
// ^^^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @param has invalid int range boundary `"bar"`: must be an integer literal, `min`, or `max`
 */
function scope(int $a){
    return $a;
}
===expect===
