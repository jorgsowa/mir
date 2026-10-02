===description===
Values named $_-prefixed, $ignored or $unused are intentionally unread
===file===
<?php
/** @return Generator<int, int> */
function generate(): Generator { yield 1; yield 2; }

foreach (generate() as $_) { echo "x"; }
foreach (generate() as $_skip) { echo "x"; }
foreach (generate() as $ignored) { echo "x"; }
foreach (generate() as $unused) { echo "x"; }
===expect===
