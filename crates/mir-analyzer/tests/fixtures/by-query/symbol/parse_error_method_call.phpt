===description===
A method call still resolves when the file has a hard parse error elsewhere.
===cursor===
symbol
===file===
<?php
class Foo { public function bar(): int { return 1; } }
function t(Foo $obj): void {
    $obj->ba<CURSOR>r();
    $x = ;
}
===expect===
kind: method call Foo::bar
type: int
