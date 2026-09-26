===description===
A cursor right after a static method name (before `(`) resolves the call, like instance method calls do.
===ignore===
===cursor===
symbol
===file===
<?php
final class B { public static function get(): int { return 1; } }
echo B::get<CURSOR>();
===expect===
kind: static call B::get
type: int
