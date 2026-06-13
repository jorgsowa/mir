===description===
NoInterfaceProperties does NOT fire for properties declared via @property on the
same sealed interface.
===file===
<?php
/**
 * @property string $name
 * @property int $age
 * @seal-properties
 */
interface Sealed {
    /** @return mixed */
    public function __get(string $key);
    /** @param mixed $value */
    public function __set(string $key, $value): void;
}

function readDeclared(Sealed $s): string {
    return $s->name;
}

function writeDeclared(Sealed $s): void {
    $s->age = 30;
}

===expect===
