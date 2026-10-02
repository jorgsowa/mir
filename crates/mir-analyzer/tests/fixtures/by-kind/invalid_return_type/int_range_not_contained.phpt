===description===
Int range not contained
===file===
<?php
/**
 * @param int<1,12> $a
 * @return int<-1, 11>
 * @suppress InvalidReturnStatement
 */
function scope(int $a){
    return $a;
//  ^^^^^^^^^^ InvalidReturnType: Return type 'int<1, 12>' is not compatible with declared 'int<-1, 11>'
}
===expect===
UnusedSuppress@7:0-7:0: Suppress annotation for 'InvalidReturnStatement' is never used
