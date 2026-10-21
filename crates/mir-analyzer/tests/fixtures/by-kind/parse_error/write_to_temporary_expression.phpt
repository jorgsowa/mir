===description===
writing a property of a new or clone expression is a parse error; variable and call bases are fine
===file===
<?php
class A { public int $p = 0; }
function f(): A { return new A(); }
$a = new A();
(new A)->p = 1;
new A()->p = 1;
(clone $a)->p = 1;
$a->p = 1;
f()->p = 1;
===expect===
ParseError@5:0-5:10: Parse error: Cannot use temporary expression in write context
ParseError@6:0-6:10: Parse error: Cannot use temporary expression in write context
ParseError@7:0-7:13: Parse error: Cannot use result of built-in function in write context
