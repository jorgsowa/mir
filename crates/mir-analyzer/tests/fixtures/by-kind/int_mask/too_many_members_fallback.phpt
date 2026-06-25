===description===
int-mask with more than 8 members would generate >256 OR-combinations. To
avoid excessive union size, mir falls back to plain `int` and accepts any integer.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param int-mask<1, 2, 4, 8, 16, 32, 64, 128, 256> $flags
 */
function set_flags(int $flags): void {}

set_flags(0);
set_flags(999); // any int accepted — no false positives from 9-member mask
===expect===
