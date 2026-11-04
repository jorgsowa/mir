===description===
OR-ed constant flags fold to a literal, so preg_match_all() types `$matches` by them
===file===
<?php
function f(string $s): int {
    preg_match_all('/a/', $s, $matches, PREG_OFFSET_CAPTURE | PREG_SET_ORDER);
    /** @mir-check $matches is list<list<array{0: string, 1: int}>> */
    return $matches[0][0][1];
}
