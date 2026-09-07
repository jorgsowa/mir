===description===
`empty($this->prop)` narrows the property, the property-receiver
counterpart of `empty($var)` — the bare-variable arm already narrows by
truthiness but the property-access arm was a no-op.
===config===
suppress=UnusedVariable,PossiblyNullPropertyAccess
===file===
<?php
class Box {
    public ?string $name = null;
}

function narrowsEmpty(Box $x): void {
    if (empty($x->name)) {
        /** @mir-check $x->name is ''|'0'|null */
        $_ = 1;
    }
}

function narrowsNotEmpty(Box $x): void {
    if (!empty($x->name)) {
        /** @mir-check $x->name is non-empty-string */
        $_ = 1;
    }
}
===expect===
