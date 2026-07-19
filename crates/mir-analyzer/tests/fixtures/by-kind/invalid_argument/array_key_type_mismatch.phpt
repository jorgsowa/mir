===description===
array<string, int> passed where array<int, int> is expected is a genuine
key-type mismatch — array_list_compatible/union_compatible only ever
compared the value type, never the key, so a definite int-vs-string key
mismatch went completely unchecked. A merely dynamic/unresolved key
(array-key, mixed) stays permissive.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<int, int> $a */
function takesIntKeyed(array $a): void {}

/** @var array<string, int> $bad */
$bad = ['x' => 1];
takesIntKeyed($bad);

/** @var array<int, int> $good */
$good = [0 => 1];
takesIntKeyed($good);
===expect===
InvalidArgument@7:14-7:18: Argument $a of takesIntKeyed() expects 'array<int, int>', got 'array<string, int>'
