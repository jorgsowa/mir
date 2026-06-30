===description===
MixedArrayOffset fires when a mixed key indexes into an inner array obtained from a typed outer access
===config===
suppress=UnusedVariable
===file===
<?php
/** @var mixed $key */
$key = 'x';
/** @var array<string, array<string, int>> $matrix */
$matrix = [];
$val = $matrix['row'][$key];
===expect===
MixedArrayOffset@6:22-6:26: Mixed type used as array offset
