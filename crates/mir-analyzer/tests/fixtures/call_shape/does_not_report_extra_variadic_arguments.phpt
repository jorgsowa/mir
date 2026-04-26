===file===
<?php
function many(int $first, int ...$rest): void {}
many(1, 2, 3);
===expect===
UnusedParam: Parameter $first is never used
UnusedParam: Parameter $rest is never used
