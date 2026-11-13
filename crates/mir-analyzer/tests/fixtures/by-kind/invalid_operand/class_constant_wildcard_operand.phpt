===description===
A `Cls::MODE_*` docblock type is the matching constants' values, so it is a valid
bitwise and arithmetic operand. An enum case ref stays an object.
===file===
<?php
final class Cache {
    public const MODE_GET = 1;
    public const MODE_PUT = 2;
    public const MODE_NORMAL = 3;
}
enum Mode { case On; }

final class Key {
    /** @param Cache::MODE_* $m */
    public function __construct(public readonly int $m = Cache::MODE_NORMAL) {}
}

function hasGet(Key $k): bool {
    return (bool) ($k->m & Cache::MODE_GET);
}

/** @param Cache::MODE_* $m */
function masks(int $m): int {
    return ($m & 1) | ($m << 2) | ($m ^ Cache::MODE_PUT);
}

/** @param Cache::MODE_* $m */
function next_mode(int $m): int {
    return $m + 1;
}

/** @param Mode::On $m */
function enumOperand(Mode $m): int {
    return $m & 1;
//         ^^^^^^ InvalidOperand: Operator '&' not supported between 'Mode::On' and '1'
}
