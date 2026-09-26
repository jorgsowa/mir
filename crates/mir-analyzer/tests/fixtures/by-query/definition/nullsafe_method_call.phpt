===description===
A nullsafe method call lands on the method.
===cursor===
definition
===file===
<?php
final class User {
    public function name(): string { return 'x'; }
}
function f(?User $u): void {
    $n = $u?->na<CURSOR>me();
}
===expect===
test.php@3:4-3:50
