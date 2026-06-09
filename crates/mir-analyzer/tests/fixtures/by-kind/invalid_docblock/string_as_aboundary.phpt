===description===
String as a boundary
===file===
<?php
/**
 * @param int<0, "bar"> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @param has invalid int range boundary `"bar"`: must be an integer literal, `min`, or `max`
