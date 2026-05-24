===description===
stringAsABoundary
===file===
<?php
/**
 * @param int<0, "bar"> $a
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidDocblock
===ignore===
TODO
