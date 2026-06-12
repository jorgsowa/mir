===description===
Unsupported reference usage continues analysis
===file===
<?php
/** @var array<string, string> */
$arr = [];

/** @var non-empty-list<string> */
$foo = ["foo"];

/** @suppress UnsupportedReferenceUsage */
$bar = &$arr[$foo[0]];

/** @trace $bar */;

===expect===
UnusedPsalmSuppress@9:0-9:0: Suppress annotation for 'UnsupportedReferenceUsage' is never used
