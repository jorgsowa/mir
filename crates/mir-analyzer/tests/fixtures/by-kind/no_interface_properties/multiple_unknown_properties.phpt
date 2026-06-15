===description===
NoInterfaceProperties fires once per distinct unknown property access on the
same sealed interface.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
/**
 * @property string $name
 * @seal-properties
 */
interface Sealed {
    /** @return mixed */
    public function __get(string $key);
    /** @param mixed $value */
    public function __set(string $key, $value): void;
}

function readMultiple(Sealed $s): void {
    $a = $s->name;
    $b = $s->age;
    $s->role = "admin";
}

===expect===
NoInterfaceProperties@15:13-15:16: Property $age is not defined on sealed interface
NoInterfaceProperties@16:4-16:22: Property $role is not defined on sealed interface
