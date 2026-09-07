===description===
A 3+ level chained array-index write (`$a['x']['y']['z'] = $v`) must nest the
shape in the order it was written (x -> y -> z) — assign_to_target's
ArrayAccess handling built `wrapped_value` by iterating the innermost keys
of `key_chain` in reverse, which transposed the two innermost segments for
any chain 3+ levels deep (2-level chains have only one key to wrap, so the
bug was invisible there).
===file===
<?php

function threeLevels(int $v): int {
    $a = [];
    $a['x']['y']['z'] = $v;
    $leaf = $a['x']['y']['z'];
    /** @mir-check $leaf is int */
    return $leaf;
}

function fourLevels(string $v): string {
    $a = [];
    $a['a']['b']['c']['d'] = $v;
    $leaf = $a['a']['b']['c']['d'];
    /** @mir-check $leaf is string */
    return $leaf;
}
===expect===
