===description===
Same unsoundness as the @pure-override check, for @mutation-free: an
override silently dropping @mutation-free lets a mutating override slip
through a caller holding an ancestor-typed reference.
===file===
<?php
interface Counter {
    /** @mutation-free */
    public function peek(): int;
}
class Mutates implements Counter {
    public int $calls = 0;
    public function peek(): int {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Mutates::peek() signature mismatch: Counter::peek() is declared @mutation-free and must be re-declared @mutation-free when overridden
        $this->calls++;
        return $this->calls;
    }
}
