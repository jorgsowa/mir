===description===
A docblock @param may narrow a multi-class union to a single concrete subclass
without emitting MethodSignatureMismatch. The native type hint (the common
supertype) is unchanged; only the docblock refines the expected type.
===config===
suppress=UnusedParam
===file===
<?php
class Vehicle {}
class Car extends Vehicle {}
class Truck extends Vehicle {}
class Bicycle extends Vehicle {}

class FleetManager {
    /** @param Car|Truck|Bicycle $vehicle */
    public function register(Vehicle $vehicle): void {}
}

class CarFleetManager extends FleetManager {
    /** @param Car $vehicle */
    public function register(Vehicle $vehicle): void {}
}
===expect===
