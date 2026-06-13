===description===
NoInterfaceProperties fires when writing to a property not in the @seal-properties
interface, even via @property-write.
===file===
<?php
/**
 * @property-write string $name
 * @seal-properties
 */
interface Sealed {
    /** @param mixed $value */
    public function __set(string $key, $value): void;
}

function setAge(Sealed $s): void {
    $s->age = 42;
}

===expect===
NoInterfaceProperties@12:5-12:17: Property $age is not defined on sealed interface
