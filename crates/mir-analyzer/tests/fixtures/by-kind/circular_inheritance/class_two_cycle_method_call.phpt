===description===
method call on a class in an inheritance cycle terminates
===file===
<?php
class A extends B { public function m(): void {} }
class B extends A {}
//<^^^^^^^^^^^^^^^^^^^^ CircularInheritance: Class B has a circular inheritance chain
function f(A $a): void { $a->m(); }
