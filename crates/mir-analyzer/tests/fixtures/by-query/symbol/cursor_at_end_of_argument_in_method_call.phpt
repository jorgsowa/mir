===description===
A cursor right after a variable argument resolves the argument, not the enclosing method call.
===cursor===
symbol
===file===
<?php
class Foo {
    public function m(Foo $a = null): Foo { return $this; }
}
function t(Foo $obj): void {
    $obj->m($obj<CURSOR>);
}
===expect===
kind: variable $obj
type: Foo
