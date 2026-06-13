===description===
NoInterfaceProperties does NOT fire when the interface is not annotated with
@seal-properties, even with @property declarations.
===file===
<?php
/**
 * @property string $name
 */
interface Unsealed {
    /** @return mixed */
    public function __get(string $key);
}

function getAny(Unsealed $u): mixed {
    return $u->anything;
}

===expect===
