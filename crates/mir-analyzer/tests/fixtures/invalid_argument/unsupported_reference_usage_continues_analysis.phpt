===description===
unsupportedReferenceUsageContinuesAnalysis
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
===ignore===
TODO
