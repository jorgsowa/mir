===description===
Phan's `associative-array` accepts a definitely non-list array with matching
string keys and value types.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param associative-array<string, int> $items
 */
function takesAssociativeArray($items): void {}

takesAssociativeArray(['x' => 1, 'y' => 2]);
===expect===
