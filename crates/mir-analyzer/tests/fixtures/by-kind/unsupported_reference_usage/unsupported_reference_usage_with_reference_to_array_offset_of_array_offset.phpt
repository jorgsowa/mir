===description===
Unsupported reference usage with reference to array offset of array offset
===ignore===
TODO
===file===
<?php
/** @var array<string, string> */
$arr = [];

/** @var non-empty-list<string> */
$foo = ["foo"];

$bar = &$arr[$foo[0]];

===expect===
