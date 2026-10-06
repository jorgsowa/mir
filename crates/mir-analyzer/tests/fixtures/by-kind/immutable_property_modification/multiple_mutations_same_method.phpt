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
//      ^^^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property width of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
        $this->height = 0.0;
//      ^^^^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property height of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
