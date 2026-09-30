===description===
Go-to-definition still resolves a method call when the file has a hard parse error elsewhere.
===cursor===
definition
===file===
<?php
class Foo { public function bar(): int { return 1; } }
function t(Foo $obj): void {
    $obj->ba<CURSOR>r();
    $x = ;
}
===expect===
test.php@2:12-2:52
