===description===
Int range not contained
===ignore===
TODO
===file===
<?php
/**
 * @param int<1,12> $a
 * @return int<-1, 11>
 * @suppress InvalidReturnStatement
 */
function scope(int $a){
    return $a;
}
===expect===
