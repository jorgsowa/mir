===description===
int-mask<4> expands to {0, 4}: the empty subset (0) and the single flag (4).
Values outside that set are rejected.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param int-mask<4> $flags
 */
function set_flags(int $flags): void {}

set_flags(0);  // valid: empty subset
set_flags(4);  // valid: the one flag
set_flags(1);  // invalid: 1 is not in {0, 4}
===expect===
InvalidArgument@9:10-9:11: Argument $flags of set_flags() expects '0|4', got '1'
