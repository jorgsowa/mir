===description===
Symbol resolves the call inside a function whose closing brace is missing.
===cursor===
symbol
===file===
<?php
class Foo { public function bar(): int { return 1; } }
function t(Foo $obj): void {
    return $obj->ba<CURSOR>r();
===expect===
kind: method call Foo::bar
type: int
