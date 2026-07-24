===description===
A `@psalm-type` alias used inside a conditional return type's branch
(`@return ($x is true ? ItemShape : int)`) never expanded — no arm in
`expand_aliases_in_atomic` recursed into `Atomic::TConditional` at all, so
it fell through to the catch-all and leaked the raw, unexpanded alias atom
name into the resolved return type.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @psalm-type ItemShape = array{id: int, name: string}
 * @param bool $wantsShape
 * @return ($wantsShape is true ? ItemShape : int)
 */
function make(bool $wantsShape) {
    if ($wantsShape) {
        return ['id' => 1, 'name' => 'x'];
    }
    return 0;
}

function testShape(): void {
    $x = make(true);
    /** @mir-check $x is array{id: int, name: string} */
    $_ = 1;
}

function testInt(): void {
    $y = make(false);
    /** @mir-check $y is int */
    $_ = 1;
}
===expect===
