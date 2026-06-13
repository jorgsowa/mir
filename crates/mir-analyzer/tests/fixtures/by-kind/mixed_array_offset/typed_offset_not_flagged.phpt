===description===
MixedArrayOffset does NOT fire when the offset has a concrete int or string type.
===config===
suppress=UnusedVariable
===file===
<?php
/** @var array<string, int> $arr */
$arr = [];
$key = "hello";
$val = $arr[$key];

===expect===
