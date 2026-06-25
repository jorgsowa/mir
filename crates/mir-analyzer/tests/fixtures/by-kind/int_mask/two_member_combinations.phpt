===description===
int-mask<2, 4> expands to {0, 2, 4, 6}. Values 1 and 3 are not combinations
of flags 2 and 4, so they are rejected.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param int-mask<2, 4> $flags
 */
function set_flags(int $flags): void {}

set_flags(0);  // valid
set_flags(2);  // valid: flag 2
set_flags(4);  // valid: flag 4
set_flags(6);  // valid: 2|4
set_flags(1);  // invalid: 1 is not a combination of 2 and 4
set_flags(3);  // invalid: 3 is not a combination of 2 and 4
===expect===
InvalidArgument@11:10-11:11: Argument $flags of set_flags() expects '0|2|4|6', got '1'
InvalidArgument@12:10-12:11: Argument $flags of set_flags() expects '0|2|4|6', got '3'
