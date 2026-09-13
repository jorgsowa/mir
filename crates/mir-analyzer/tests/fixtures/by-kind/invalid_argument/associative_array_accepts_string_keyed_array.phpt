===description===
`associative-array` accepts matching string-keyed arrays.
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
