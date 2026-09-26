===description===
A nullsafe call resolves the method on the non-null receiver type.
===cursor===
symbol
===file===
<?php
final class User {
    public function name(): string { return 'x'; }
}
function f(?User $u): void {
    $n = $u?->na<CURSOR>me();
}
===expect===
kind: method call User::name
type: string|null
