===description===
MixedArrayAccess does NOT fire when the array has a concrete element type.
===config===
suppress=UnusedVariable
===file===
<?php
/** @var array<int, string> $arr */
$arr = [];
$val = $arr[0];

===expect===
