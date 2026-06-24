===description===
Methods that only read $this properties in a @psalm-immutable class do not trigger
ImmutablePropertyModification.
===config===
suppress=UnusedParam
===file===
<?php

/** @psalm-immutable */
class Color {
    public function __construct(
        public int $r,
        public int $g,
        public int $b,
    ) {}

    public function toHex(): string {
        return sprintf('#%02x%02x%02x', $this->r, $this->g, $this->b);
    }

    public function withRed(int $r): self {
        return new self($r, $this->g, $this->b);
    }
}
===expect===
