===description===
A @psalm-mutation-free method may call other @psalm-mutation-free methods on $this
without any error — chaining read-only operations is safe.
===file===
<?php

class Rectangle {
    public function __construct(
        public float $width,
        public float $height,
    ) {}

    /** @psalm-mutation-free */
    public function area(): float {
        return $this->width * $this->height;
    }

    /** @psalm-mutation-free */
    public function perimeter(): float {
        return 2.0 * ($this->width + $this->height);
    }

    /** @psalm-mutation-free */
    public function describe(): string {
        return "area=" . $this->area() . " perimeter=" . $this->perimeter();
    }
}
===expect===
