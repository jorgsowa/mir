===description===
does not report extra variadic arguments
===config===
suppress=UnusedParam
===file===
<?php
function many(int $first, int ...$rest): void {}
many(1, 2, 3);
===expect===
