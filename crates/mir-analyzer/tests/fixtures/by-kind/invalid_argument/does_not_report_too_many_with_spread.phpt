===description===
does not report too many with spread
===config===
suppress=UnusedParam
===file===
<?php
function takes_one(int $a): void {}
$arr = [1, 2, 3];
takes_one(...$arr);
===expect===
