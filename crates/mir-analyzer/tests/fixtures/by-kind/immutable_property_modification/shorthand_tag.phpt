===description===
The shorter @immutable tag (without the psalm- prefix) is also recognized.
===file===
<?php

/** @immutable */
class Pixel {
    public function __construct(public int $x, public int $y) {}

    public function translate(int $dx, int $dy): void {
        $this->x += $dx;
        $this->y += $dy;
    }
}
===expect===
ImmutablePropertyModification@8:8-8:23: Assigning to property x of $this in a @psalm-immutable class
ImmutablePropertyModification@9:8-9:23: Assigning to property y of $this in a @psalm-immutable class
