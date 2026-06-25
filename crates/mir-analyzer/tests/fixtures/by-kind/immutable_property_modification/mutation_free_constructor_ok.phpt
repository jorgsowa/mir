===description===
A constructor annotated @psalm-mutation-free may still assign to $this properties —
initialization in the constructor is not a mutation (same exemption as @psalm-immutable).
===file===
<?php

class ImmutablePoint {
    public float $x;
    public float $y;

    /** @psalm-mutation-free */
    public function __construct(float $x, float $y) {
        $this->x = $x;
        $this->y = $y;
    }

    /** @psalm-mutation-free */
    public function translate(float $dx, float $dy): void {
        $this->x += $dx;
        $this->y += $dy;
    }
}
===expect===
ImmutablePropertyModification@15:8-15:23: Assigning to property x of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
ImmutablePropertyModification@16:8-16:23: Assigning to property y of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
