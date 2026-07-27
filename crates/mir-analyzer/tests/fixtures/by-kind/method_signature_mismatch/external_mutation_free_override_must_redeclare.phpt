===description===
Same unsoundness as the @pure-override check, for @external-mutation-free:
an override silently dropping @external-mutation-free lets a
this-mutating override slip through a caller holding an ancestor-typed
reference.
===file===
<?php
interface Cache {
    /** @external-mutation-free */
    public function bump(): int;
}
class Mutates implements Cache {
    public int $calls = 0;
    public function bump(): int {
        $this->calls++;
        return $this->calls;
    }
}
===expect===
MethodSignatureMismatch@8:4-8:33: Method Mutates::bump() signature mismatch: Cache::bump() is declared @external-mutation-free and must be re-declared @external-mutation-free when overridden
