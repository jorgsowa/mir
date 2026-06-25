===description===
int-mask<1, 2, 4> expands to all 8 OR-combinations {0,1,2,3,4,5,6,7}.
Passing any of those values is accepted without error.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param int-mask<1, 2, 4> $flags
 */
function set_flags(int $flags): void {}

set_flags(0);  // no flags
set_flags(1);  // flag A
set_flags(2);  // flag B
set_flags(3);  // A|B
set_flags(4);  // flag C
set_flags(5);  // A|C
set_flags(6);  // B|C
set_flags(7);  // A|B|C (all set)
===expect===
