===source===
<?php
function f(int ...$xs): void { var_dump($xs); }
function test(): void { f('a'); }
===expect===
InvalidArgument: Argument $xs of f() expects 'int', got '"a"'
