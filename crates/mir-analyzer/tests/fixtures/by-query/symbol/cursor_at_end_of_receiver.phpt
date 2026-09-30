===description===
A cursor right after a method call's receiver resolves the receiver, not the call.
===cursor===
symbol
===file===
<?php
class Foo {
    public function m(Foo $a = null): Foo { return $this; }
}
function t(Foo $obj): void {
    $obj<CURSOR>->m();
}
===expect===
kind: variable $obj
type: Foo
