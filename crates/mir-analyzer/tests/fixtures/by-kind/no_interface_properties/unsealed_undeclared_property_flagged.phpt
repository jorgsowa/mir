===description===
NoInterfaceProperties fires on a plain interface even without @seal-properties —
real PHP semantics: interfaces can't declare properties at all, so any access
through an interface type not covered by @property is suspect regardless of
whether the interface opted into sealing. Presence of __get doesn't exempt it
either (unlike the class-side UndefinedProperty check).
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
NoInterfaceProperties@11:15-11:23: Property $anything is not defined on this interface
