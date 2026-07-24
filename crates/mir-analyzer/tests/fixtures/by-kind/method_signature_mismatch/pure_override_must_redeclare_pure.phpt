===description===
A @pure ancestor method establishes a contract that call/method.rs's
purity/immutability safety checks rely on when a call resolves through
the ancestor's (or an interface's) static type — an override that
silently drops @pure without re-declaring it made that already-shipped
enforcement unsound for any caller typed as the ancestor.
===file===
<?php
interface Calculator {
    /** @pure */
    public function add(int $a, int $b): int;
}
class Impure implements Calculator {
    public int $calls = 0;
    public function add(int $a, int $b): int {
        $this->calls++;
        return $a + $b;
    }
}
===expect===
MethodSignatureMismatch@8:4-8:46: Method Impure::add() signature mismatch: Calculator::add() is declared @pure and must be re-declared @pure when overridden
