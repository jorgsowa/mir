===description===
Calling a @pure method on $this inside a @psalm-immutable class is allowed —
@pure implies no side effects and no mutation, so it satisfies the immutability
contract.
===file===
<?php

/** @psalm-immutable */
class Temperature {
    public function __construct(public float $celsius) {}

    /** @pure */
    public function toCelsius(): float {
        return $this->celsius;
    }

    public function describe(): string {
        return $this->toCelsius() . "°C";
    }
}
===expect===
