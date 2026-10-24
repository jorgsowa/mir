===description===
writing a property of a new or clone expression is a parse error; variable and call bases are fine
===file===
<?php
class A { public int $p = 0; }
function f(): A { return new A(); }
$a = new A();
(new A)->p = 1;
//<^^^^^^^^^^ CompileError: Compile error: Cannot use temporary expression in write context [temporary-expression-write]
new A()->p = 1;
//<^^^^^^^^^^ CompileError: Compile error: Cannot use temporary expression in write context [temporary-expression-write]
(clone $a)->p = 1;
//<^^^^^^^^^^^^^ CompileError: Compile error: Cannot use result of built-in function in write context [builtin-result-write]
$a->p = 1;
f()->p = 1;
