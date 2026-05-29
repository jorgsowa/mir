===description===
reports variadic wrong type
===file===
<?php
function f(int ...$xs): void { var_dump($xs); }
function test(): void { f('a'); }
===expect===
InvalidArgument@3:27-3:30: Argument $xs of f() expects 'int', got '"a"'
