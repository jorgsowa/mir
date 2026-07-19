===description===
`isset($this->prop['key'])` / `!empty($this->prop['key'])` strip
null/false from the property container itself, the property-receiver
counterpart of the existing plain-variable `isset($base['key'])` handling
— `array_access_base_var` only ever recognized a plain variable base.
===config===
suppress=UnusedVariable,PossiblyNullPropertyAccess,PossiblyInvalidArrayAccess,MissingPropertyType
===file===
<?php
class Box {
    /** @var array<string, int>|false|null */
    public $data = null;
}

function issetNarrowsContainer(Box $x): void {
    if (isset($x->data['key'])) {
        /** @mir-check $x->data is array<string, int> */
        $_ = 1;
    }
}

function notEmptyNarrowsContainer(Box $x): void {
    if (!empty($x->data['key'])) {
        /** @mir-check $x->data is array<string, int> */
        $_ = 1;
    }
}
===expect===
