===description===
Static methods of a @psalm-immutable class have no $this — no ImmutablePropertyModification.
===config===
suppress=UnusedParam
===file===
<?php

/** @psalm-immutable */
class Temperature {
    public function __construct(public float $celsius) {}

    public static function fromFahrenheit(float $f): self {
        return new self(($f - 32.0) * 5.0 / 9.0);
    }
}
===expect===
