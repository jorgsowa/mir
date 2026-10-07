===description===
A helper of a @psalm-immutable class is itself checked as mutation-free, so a
mutation is reported where it happens, not at the `$this->helper()` call.
===file===
<?php

/** @psalm-immutable */
class Point {
    public function __construct(
        public float $x,
        public float $y,
    ) {}

    public function reset(): void {
        $this->doReset();
    }

    private function doReset(): void {
        $this->x = 0.0;
//      ^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property x of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
        $this->y = 0.0;
//      ^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property y of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
