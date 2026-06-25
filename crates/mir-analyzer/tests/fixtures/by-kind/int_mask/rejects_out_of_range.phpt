===description===
int-mask<1, 2, 4> rejects a literal integer that cannot be formed by OR-ing
any subset of {1, 2, 4}: 8 is out of range.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param int-mask<1, 2, 4> $flags
 */
function set_flags(int $flags): void {}

set_flags(8);
===expect===
InvalidArgument@7:10-7:11: Argument $flags of set_flags() expects '0|1|2|3|4|5|6|7', got '8'
