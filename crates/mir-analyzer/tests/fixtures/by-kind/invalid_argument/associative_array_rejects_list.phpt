===description===
`associative-array` rejects lists.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param associative-array<string, int> $items
 */
function takesAssociativeArray($items): void {}

takesAssociativeArray([1, 2, 3]);
//                    ^^^^^^^^^ InvalidArgument: Argument $items of takesAssociativeArray() expects 'array<string, int>&array{}', got 'array{0: 1, 1: 2, 2: 3}'
===expect===
