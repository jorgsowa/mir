===file===
<?php
function takes_one(int $a): void {}
$arr = [1, 2, 3];
takes_one(...$arr);
===expect===
UnusedParam: Parameter $a is never used
