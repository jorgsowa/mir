===description===
Unsupported reference usage continues analysis
===config===
suppress=Trace,UnusedVariable
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
