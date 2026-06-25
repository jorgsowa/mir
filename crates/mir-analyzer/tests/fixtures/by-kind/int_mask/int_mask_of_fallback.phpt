===description===
int-mask-of<T::*> cannot resolve class constants at parse time, so it falls
back to plain `int`. Any integer value is accepted without false-positive errors.
===config===
suppress=UnusedParam
===file===
<?php
class Flags {
    const FLAG_A = 1;
    const FLAG_B = 2;
    const FLAG_C = 4;
}

/**
 * @param int-mask-of<Flags::*> $flags
 */
function set_flags(int $flags): void {}

set_flags(0);
set_flags(1);
set_flags(8);    // not a real combination, but int-mask-of falls back to int
set_flags(999);  // any int is accepted
===expect===
