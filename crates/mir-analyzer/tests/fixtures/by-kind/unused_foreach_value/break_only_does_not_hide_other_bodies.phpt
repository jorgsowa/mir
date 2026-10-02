===description===
Only a bare-break body or a conventional name is exempt; other unread values are still reported
===file===
<?php
/** @return Generator<int, int> */
function generate(): Generator { yield 1; yield 2; }

foreach (generate() as $a) { echo "x"; }
//                     ^^ UnusedForeachValue: Foreach value $a is never read

foreach (generate() as $b) { if (rand(0, 1)) { break; } }
//                     ^^ UnusedForeachValue: Foreach value $b is never read

foreach (generate() as $c) { break; echo "x"; }
//                     ^^ UnusedForeachValue: Foreach value $c is never read
//                                  ^^^^^^^^^ UnreachableCode: Unreachable code detected

foreach (generate() as $ignoredValue) { echo "x"; }
//                     ^^^^^^^^^^^^^ UnusedForeachValue: Foreach value $ignoredValue is never read
===expect===
