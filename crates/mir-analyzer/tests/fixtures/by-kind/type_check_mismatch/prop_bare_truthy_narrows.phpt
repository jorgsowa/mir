===description===
Bare truthy `if ($this->prop)` narrows the property, the property-receiver
counterpart of the plain-variable truthy/falsy fallback — `narrow_prop_loose_bool`
already existed for `==`/`!=` but was never wired into this catch-all arm.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
final class Holder {
    /** @var bool */
    public $flag = false;

    public function narrowsTrue(): void {
        if ($this->flag) {
            /** @mir-check $this->flag is true */
            $_ = 1;
        }
    }

    public function narrowsFalse(): void {
        if (!$this->flag) {
            /** @mir-check $this->flag is false */
            $_ = 1;
        }
    }
}
===expect===
