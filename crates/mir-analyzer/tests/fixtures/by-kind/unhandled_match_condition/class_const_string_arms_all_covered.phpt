===description===
UnhandledMatchCondition does NOT fire when class-constant arm conditions cover every string literal.
===file===
<?php
class C {
    const A = 'a';
    const B = 'b';
}
/** @param 'a'|'b' $x */
function f(string $x): string {
    return match ($x) {
        C::A => 'x',
        C::B => 'y',
    };
}
===expect===
