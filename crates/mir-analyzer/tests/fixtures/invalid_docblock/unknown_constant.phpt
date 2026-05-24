===description===
unknownConstant
===file===
<?php
/**
 * @param int<0, FOO> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock
===ignore===
TODO
