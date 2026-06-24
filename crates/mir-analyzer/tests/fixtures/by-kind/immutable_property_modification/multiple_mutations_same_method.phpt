===description===
Each $this->prop assignment in a @psalm-immutable method is reported individually.
===file===
<?php

/** @psalm-immutable */
class Rect {
    public function __construct(
        public float $width,
        public float $height,
    ) {}

    public function reset(): void {
        $this->width = 0.0;
        $this->height = 0.0;
    }
}
===expect===
ImmutablePropertyModification@11:8-11:26: Assigning to property width of $this in a @psalm-immutable class
ImmutablePropertyModification@12:8-12:27: Assigning to property height of $this in a @psalm-immutable class
