===description===
Unsupported reference usage with reference to array offset of array offset
===config===
suppress=UnusedVariable
===file===
<?php
/** @var array<string, string> */
$arr = [];

/** @var non-empty-list<string> */
$foo = ["foo"];

$bar = &$arr[$foo[0]];

===expect===
UnsupportedReferenceUsage@8:1-8:22: Reference assignment is not supported
