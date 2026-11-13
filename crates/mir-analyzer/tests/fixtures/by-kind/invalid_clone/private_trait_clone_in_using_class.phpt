===description===
A private `__clone()` from a trait belongs to the class using the trait, so
`clone $this` inside that class is allowed. Cloning from outside still fails.
===file===
<?php
trait NoClone {
    private function __clone() {}
}

final class Loc {
    use NoClone;
    public ?int $line = null;

    public function withLine(?int $l): self {
        $c = clone $this;
        $c->line = $l;
        return $c;
    }
}

function copy(Loc $l): Loc {
    return clone $l;
//         ^^^^^^^^ InvalidClone: Call to private method NoClone::__clone() from global scope
}
