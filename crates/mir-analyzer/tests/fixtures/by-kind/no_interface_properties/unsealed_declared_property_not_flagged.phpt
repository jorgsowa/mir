===description===
NoInterfaceProperties does NOT fire for a property declared via @property on a
plain interface, even without @seal-properties — sealing only ever narrowed
which unknown accesses were rejected, it was never what legitimized a known one.
===file===
<?php
/**
 * @property string $name
 */
interface Unsealed {
    /** @return mixed */
    public function __get(string $key);
}

function getName(Unsealed $u): string {
    return $u->name;
}

===expect===
