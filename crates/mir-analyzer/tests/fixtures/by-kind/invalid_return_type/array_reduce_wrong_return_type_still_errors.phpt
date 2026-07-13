===description===
Sibling of array_reduce_infers_return_type: a genuinely wrong declared return type still errors.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param list<int> $ints
 * @return string
 */
function sumInts(array $ints): string {
    return array_reduce($ints, fn(int $c, int $x): int => $c + $x, 0);
}
===expect===
InvalidReturnType@7:4-7:70: Return type 'int' is not compatible with declared 'string'
