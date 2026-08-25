===description===
Phan's `associative-array` is distinct from a list, so a plain list argument
is rejected even though its values match.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param associative-array<string, int> $items
 */
function takesAssociativeArray($items): void {}

takesAssociativeArray([1, 2, 3]);
===expect===
InvalidArgument@7:22-7:31: Argument $items of takesAssociativeArray() expects 'array<string, int>&array{}', got 'array{0: 1, 1: 2, 2: 3}'
