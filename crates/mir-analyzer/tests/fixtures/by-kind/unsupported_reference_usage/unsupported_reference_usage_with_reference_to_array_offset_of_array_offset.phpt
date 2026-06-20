===description===
Reference to array offset of array offset does not fire UnsupportedReferenceUsage.
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
