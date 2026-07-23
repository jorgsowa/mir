===description===
Calling a provably pure (or mutation-free) method on a typed parameter
inside a @pure function is not a purity violation — the callee's own
resolved purity clears it, even though the receiver is a parameter.
===file===
<?php
namespace Qux;

class Reader {
    public int $a = 5;

    /** @pure */
    public function double(): int {
        return $this->a * 2;
    }

    /** @psalm-mutation-free */
    public function triple(): int {
        return $this->a * 3;
    }
}

/** @pure */
function useDouble(Reader $r): int {
    return $r->double();
}

/** @pure */
function useTriple(Reader $r): int {
    return $r->triple();
}
===expect===
