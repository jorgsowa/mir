===description===
ImmutablePropertyModification fires when a non-constructor method of a
@psalm-immutable class assigns to $this->prop.
===file===
<?php

/** @psalm-immutable */
class Point {
    public function __construct(
        public float $x,
        public float $y,
    ) {}

    public function mutate(): void {
        $this->x = 0.0;
    }
}
===expect===
ImmutablePropertyModification@11:8-11:22: Assigning to property x of $this in a @psalm-immutable class
