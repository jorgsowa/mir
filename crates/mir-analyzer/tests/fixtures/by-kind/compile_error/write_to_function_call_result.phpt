===description===
A function call result cannot be assigned to.
===file===
<?php
function f(): int { return 1; }
f() = 2;
//<^^^ CompileError: Can't use function return value in write context [call-result-write]
