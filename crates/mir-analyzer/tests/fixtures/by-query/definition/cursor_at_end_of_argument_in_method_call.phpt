===description===
A cursor right after a variable argument in a method call does not resolve to the method.
===cursor===
definition
===file===
<?php
class Foo {
    public function m(Foo $a = null): Foo { return $this; }
}
function t(Foo $obj): void {
    $obj->m($obj<CURSOR>);
}
===expect===
error: NotFound
