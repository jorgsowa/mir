===description===
maxSpecifiedAsFirst
===file===
<?php
/**
 * @param int<max, 0> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock
===ignore===
TODO
