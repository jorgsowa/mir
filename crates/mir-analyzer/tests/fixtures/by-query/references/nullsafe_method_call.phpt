===description===
Nullsafe calls count as method references.
===cursor===
references
===file===
<?php
final class User {
    public function name(): string { return 'x'; }
}
function f(?User $u, User $v): void {
    $u?->na<CURSOR>me();
    $v->name();
}
===expect===
test.php@6:9-6:13
test.php@7:8-7:12
