===description===
@psalm-mutation-free fires for every $this->prop assignment in the method, not
just the first.
===file===
<?php

class Vector {
    public float $x;
    public float $y;
    public float $z;

    public function __construct(float $x, float $y, float $z) {
        $this->x = $x;
        $this->y = $y;
        $this->z = $z;
    }

    /** @psalm-mutation-free */
    public function zero(): void {
        $this->x = 0.0;
        $this->y = 0.0;
        $this->z = 0.0;
    }
}
===expect===
ImmutablePropertyModification@16:8-16:22: Assigning to property x of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
ImmutablePropertyModification@17:8-17:22: Assigning to property y of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
ImmutablePropertyModification@18:8-18:22: Assigning to property z of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
