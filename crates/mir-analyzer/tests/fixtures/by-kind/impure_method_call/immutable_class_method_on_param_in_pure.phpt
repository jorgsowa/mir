===description===
Methods of a `@psalm-immutable` class are mutation-free, so a pure function may call them
on a parameter; a mutable class's method is still impure.
===file===
<?php
/** @psalm-immutable */
final class U {
    public function __construct(public bool $p = false) {}
    public function get(): bool { return $this->p; }
    public function with(bool $p): self { $c = clone $this; $c->p = $p; return $c; }
}

final class M {
    public bool $p = false;
    public function get(): bool { return $this->p; }
    public function set(): void { $this->p = true; }
}

/** @psalm-pure */
function f(U $u, M $m): array {
    return [$u->get(), $u->with(true), $m->set()];
//                                     ^^^^^^^^^ ImpureMethodCall: Calling impure method set() in a pure or immutable context
}
