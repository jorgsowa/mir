===description===
In a non-strict file, `int|float` (from `hexdec()` or a division) passed to an `int` parameter is PossiblyInvalidArgument; a value that is always a float stays ImplicitFloatToIntCast.
===file===
<?php
function partial(string $hex): string {
    return chr(hexdec($hex));
//             ^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $codepoint of chr() expects 'int', possibly different type 'int|float' provided
}

/** @param int $width */
function bar($width): int { return $width; }

function divided(int $total, int $parts): int {
    return bar($total / $parts);
//             ^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $width of bar() expects 'int', possibly different type 'int|float' provided
}

function always(float $f): string {
    return chr($f);
//             ^^ ImplicitFloatToIntCast: Implicit cast from float to int truncates the fractional part
}
