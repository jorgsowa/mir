===description===
An array literal argument passed where a scalar param is expected is
flagged — previously array_list_compatible treated any TKeyedArray
argument as compatible with anything, silencing this entirely
===config===
suppress=UnusedParam
===file===
<?php
function needsInt(int $x): void {}
needsInt([1, 2, 3]);
===expect===
InvalidArgument@3:9-3:18: Argument $x of needsInt() expects 'int', got 'array{0: 1, 1: 2, 2: 3}'
