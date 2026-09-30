===description===
Definition resolves in a file truncated mid-call while typing (unclosed call, function and file).
===cursor===
definition
===file===
<?php
class Foo { public function bar(int $n): int { return $n; } }
function t(Foo $obj): void {
    $obj->ba<CURSOR>r(
===expect===
test.php@2:12-2:59
