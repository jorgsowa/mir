===description===
does not report too many with spread
===file===
<?php
function takes_one(int $a): void {}
$arr = [1, 2, 3];
takes_one(...$arr);
===expect===
UnusedParam@2:20: Parameter $a is never used
