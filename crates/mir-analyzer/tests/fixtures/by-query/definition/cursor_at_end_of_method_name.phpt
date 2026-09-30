===description===
A cursor right after a method name resolves that method.
===cursor===
definition
===file===
<?php
class Foo {
    public function m(Foo $a = null): Foo { return $this; }
}
function t(Foo $obj): void {
    $obj->m<CURSOR>();
}
===expect===
test.php@3:4-3:59
