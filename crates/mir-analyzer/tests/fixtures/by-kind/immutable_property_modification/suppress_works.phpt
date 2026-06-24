===description===
@suppress ImmutablePropertyModification silences the immutable property assignment diagnostic.
===file===
<?php

/** @psalm-immutable */
class LazyPoint {
    private ?float $cachedMag = null;

    public function __construct(public float $x, public float $y) {}

    public function magnitude(): float {
        if ($this->cachedMag === null) {
            /** @suppress ImmutablePropertyModification */
            $this->cachedMag = sqrt($this->x ** 2 + $this->y ** 2);
        }
        return $this->cachedMag;
    }
}
===expect===
