===description===
Float as a boundary
===file===
<?php
/**
 * @param int<0, 5.5> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @param has invalid int range boundary `5.5`: must be an integer literal, `min`, or `max`
