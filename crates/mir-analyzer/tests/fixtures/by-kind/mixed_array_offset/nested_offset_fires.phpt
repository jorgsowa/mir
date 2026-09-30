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
//                    ^^^^ MixedArrayOffset: Mixed type used as array offset
===expect===
