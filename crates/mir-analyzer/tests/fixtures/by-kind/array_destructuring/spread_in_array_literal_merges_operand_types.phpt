===description===
Spreading two string-keyed shapes into a literal (`[...$x, ...$y]`) merges
into a precise shape with both operands' keys, not a generic
`array<string, int|string>` (let alone `array<mixed, mixed>`).
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param array{a: int} $x
 * @param array{b: string} $y
 */
function test(array $x, array $y): void {
    $z = [...$x, ...$y];
    /** @mir-check $z is array{a: int, b: string} */
    echo 1;
}
===expect===
