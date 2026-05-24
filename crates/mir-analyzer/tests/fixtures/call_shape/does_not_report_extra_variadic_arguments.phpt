===description===
does not report extra variadic arguments
===file===
<?php
function many(int $first, int ...$rest): void {}
many(1, 2, 3);
===expect===
UnusedParam@2:15: Parameter $first is never used
UnusedParam@2:27: Parameter $rest is never used
