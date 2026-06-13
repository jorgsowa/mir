===description===
Unknown constant
===config===
suppress=MissingReturnType
===file===
<?php
/**
 * @param int<0, FOO> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @param has invalid int range boundary `FOO`: must be an integer literal, `min`, or `max`
