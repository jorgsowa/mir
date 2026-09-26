===description===
A method call on a union receiver resolves to the first member's method.
===cursor===
symbol
===file===
<?php
interface Named { public function name(): string; }
final class A implements Named { public function name(): string { return 'a'; } }
final class B implements Named { public function name(): string { return 'b'; } }
function f(A|B $x): string {
    return $x->na<CURSOR>me();
}
===expect===
kind: method call A::name
type: string
