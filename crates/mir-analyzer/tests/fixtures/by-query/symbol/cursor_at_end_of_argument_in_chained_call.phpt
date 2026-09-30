===description===
A cursor right after an argument in a chained call resolves the argument.
===cursor===
symbol
===file===
<?php
class Foo {
    public function m(Foo $a = null): Foo { return $this; }
}
function t(Foo $obj): void {
    $obj->m()->m($obj<CURSOR>)->m();
}
===expect===
kind: variable $obj
type: Foo
