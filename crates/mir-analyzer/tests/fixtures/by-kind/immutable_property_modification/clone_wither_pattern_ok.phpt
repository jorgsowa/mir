===description===
Writing to a fresh `clone $this` before returning it (the standard
immutable "wither" idiom) must stay unflagged — the clone is a new,
unaliased object nothing else can observe yet. Contrast with a write
through a genuinely external receiver (a parameter), which must still be
flagged — the exemption must not blanket-disable the check.
===file===
<?php
/** @psalm-immutable */
class Point {
    public function __construct(
        public float $x,
        public float $y,
    ) {}

    public function withX(float $x): self {
        $clone = clone $this;
        $clone->x = $x;
        return $clone;
    }

    public function mutateExternal(self $other): void {
        $other->x = 1.0;
    }
}
===expect===
ImmutablePropertyModification@16:8-16:23: Assigning to property x of $other in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
