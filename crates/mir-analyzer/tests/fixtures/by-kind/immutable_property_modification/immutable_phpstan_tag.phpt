===description===
@phpstan-immutable is recognized as an alias for @psalm-immutable.
===file===
<?php

/** @phpstan-immutable */
class Pixel {
    public function __construct(public int $x, public int $y) {}

    public function translate(int $dx, int $dy): void {
        $this->x += $dx;
        $this->y += $dy;
    }
}
===expect===
ImmutablePropertyModification@8:8-8:23: Assigning to property x of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
ImmutablePropertyModification@9:8-9:23: Assigning to property y of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
