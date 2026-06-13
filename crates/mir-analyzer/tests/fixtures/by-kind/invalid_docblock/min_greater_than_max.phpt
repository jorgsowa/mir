===description===
Min greater than max
===config===
suppress=MissingReturnType
===file===
<?php
/**
 * @param int<4, 3> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @param has invalid int range: min (4) must not be greater than max (3)
