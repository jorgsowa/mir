===description===
A small bounded int range is accepted where the equivalent literal union is expected; a wider or oversized range is not.
===file===
<?php
declare(strict_types=1);

class Holder {
    /** @var -1|0|1 */
    public int $sign = 0;
    /** @var 0|1|2 */
    public int $small = 0;
    /** @var 0|1 */
    public int $flag = 0;
}

function spaceship(Holder $h, int $a, int $b): void {
    $cmp = $a <=> $b;
    /** @mir-check $cmp is int<-1, 1> */
    $h->sign = $cmp;
}

/** @param int<0, 2> $r */
function exactRange(Holder $h, int $r): void {
    /** @mir-check $r is int<0, 2> */
    $h->small = $r;
}

/** @param int<0, 3> $r */
function rangeTooWide(Holder $h, int $r): void {
    $h->small = $r;
}

/** @param int<0, 100> $r */
function rangeTooLarge(Holder $h, int $r): void {
    $h->flag = $r;
}

/** @param int<0, 1>|int<5, 5> $r */
function unionOfRanges(Holder $h, int $r): void {
    $h->flag = $r;
}
===expect===
InvalidPropertyAssignment@27:4-27:18: Property $small expects '0|1|2', cannot assign 'int<0, 3>'
InvalidPropertyAssignment@32:4-32:17: Property $flag expects '0|1', cannot assign 'int<0, 100>'
InvalidPropertyAssignment@37:4-37:17: Property $flag expects '0|1', cannot assign 'int<0, 1>|int<5, 5>'
