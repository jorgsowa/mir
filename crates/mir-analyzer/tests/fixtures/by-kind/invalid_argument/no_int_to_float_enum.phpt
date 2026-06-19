===description===
No int to float enum
===config===
suppress=UnusedParam
===file===
<?php
/** @param 0.3|0.5 $p */
function f($p): void {}
f(1);
===expect===
InvalidArgument@4:2-4:3: Argument $p of f() expects '0.3|0.5', got '1'
