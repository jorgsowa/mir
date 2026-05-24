===description===
unsupportedReferenceUsageContinuesAnalysis
===file===
<?php
/** @var array<string, string> */
$arr = [];

/** @var non-empty-list<string> */
$foo = ["foo"];

/** @psalm-suppress UnsupportedReferenceUsage */
$bar = &$arr[$foo[0]];

/** @psalm-trace $bar */;

===expect===
===ignore===
TODO
