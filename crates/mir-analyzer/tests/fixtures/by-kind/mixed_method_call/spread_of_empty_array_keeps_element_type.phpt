===description===
Spreading an empty array into a literal contributes no element type, so a later spread of a typed array keeps its element type.
===file===
<?php
final class Foo {
    public function m(): int { return 1; }
}

/** @param array<string, Foo> $b */
function f(array $b): void {
    $a = [];
    foreach ([...$a, ...$b] as $x) {
        /** @mir-check $x is Foo */
        $x->m();
    }
}
