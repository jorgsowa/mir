===description===
Spreading typed arrays into a literal (`[...$x, ...$y]`) must merge the
operands' actual key/value types, not collapse the whole literal to
`array<mixed, mixed>`.
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
    /** @mir-check $z is array<string, int|string> */
    echo 1;
}
===expect===
