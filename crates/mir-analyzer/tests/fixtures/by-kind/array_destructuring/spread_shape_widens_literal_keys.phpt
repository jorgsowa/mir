===description===
Spreading a shape into a new array literal (`[...$a]`) widens the key
domain to `string`/`int`, same as any other path that folds a shape's
properties into a generic array's key type — not each key kept as its own
literal.
===config===
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $a = ['x' => 1, 'y' => 2];
    $merged = [...$a];
    /** @mir-check $merged is array<string, 1|2> */
    $_ = $merged;
}
===expect===
