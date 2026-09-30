===description===
A cursor right after a method name still resolves the method call.
===cursor===
symbol
===file===
<?php
class Foo {
    public function m(Foo $a = null): Foo { return $this; }
}
function t(Foo $obj): void {
    $obj->m<CURSOR>();
}
===expect===
kind: method call Foo::m
type: Foo
