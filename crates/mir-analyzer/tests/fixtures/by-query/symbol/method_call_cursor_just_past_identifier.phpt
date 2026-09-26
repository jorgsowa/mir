===description===
A cursor right after a method name (before `(`) resolves through the call expression span.
===cursor===
symbol
===file===
<?php
final class B { public function get(): int { return 1; } }
echo (new B())->get<CURSOR>();
===expect===
kind: method call B::get
type: int
