===description===
reports variadic wrong type
===config===
suppress=ForbiddenCode
===file===
<?php
function f(int ...$xs): void { var_dump($xs); }
function test(): void { f('a'); }
===expect===
InvalidArgument@3:26-3:29: Argument $xs of f() expects 'int', got '"a"'
