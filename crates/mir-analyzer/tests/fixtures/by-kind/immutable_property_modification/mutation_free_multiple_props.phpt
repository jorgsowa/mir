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
//      ^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property x of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
        $this->y = 0.0;
//      ^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property y of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
        $this->z = 0.0;
//      ^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property z of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
