===description===
Max specified as first
===file===
<?php
/**
 * @param int<max, 0> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @param has invalid int range: `max` must be the second argument, not the first
