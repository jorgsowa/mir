===description===
`is_int($this->prop) || is_string($this->prop)` narrows the property to
int|string — the property-receiver counterpart of or_type_fn_narrows_to_union,
which only wired the var-side narrow_type_fn_disjuncts into narrow_or_instanceof_true.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
final class Holder {
    /** @var int|string|bool */
    public $prop;

    public function narrows(): void {
        if (is_int($this->prop) || is_string($this->prop)) {
            /** @mir-check $this->prop is int|string */
            $_ = 1;
        }
    }
}
===expect===
