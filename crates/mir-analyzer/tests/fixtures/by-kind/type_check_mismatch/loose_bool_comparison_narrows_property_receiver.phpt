===description===
`$this->prop == true`/`== false` narrows the property to truthy/falsy, the
property-receiver counterpart of narrow_var_loose_bool (which only handled a
plain variable receiver).
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
final class Holder {
    /** @var int|string */
    public $value;

    public function narrowsLooseEqualTrue(): void {
        if ($this->value == true) {
            /** @mir-check $this->value is int|non-empty-string */
            $_ = 1;
        }
    }

    public function narrowsLooseEqualFalse(): void {
        if ($this->value == false) {
            /** @mir-check $this->value is 0|""|"0" */
            $_ = 1;
        }
    }
}
===expect===
