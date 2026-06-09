===description===
Min specified as second
===file===
<?php
/**
 * @param int<0, min> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @param has invalid int range: `min` must be the first argument, not the second
