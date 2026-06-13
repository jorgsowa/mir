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
