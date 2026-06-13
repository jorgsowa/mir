===description===
NoInterfaceProperties fires when reading a property not declared via @property
on a @seal-properties interface.
===file===
<?php
/**
 * @property-read string $name
 * @seal-properties
 */
interface Sealed {
    /** @return mixed */
    public function __get(string $key);
}

function getAge(Sealed $s): mixed {
    return $s->age;
}

===expect===
NoInterfaceProperties@12:16-12:19: Property $age is not defined on sealed interface
