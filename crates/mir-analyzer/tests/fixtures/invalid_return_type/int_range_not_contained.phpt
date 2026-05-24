===description===
intRangeNotContained
===file===
<?php
/**
 * @param int<1,12> $a
 * @return int<-1, 11>
 * @psalm-suppress InvalidReturnStatement
 */
function scope(int $a){
    return $a;
}
===expect===
InvalidReturnType
===ignore===
TODO
