===description===
narrow_prop_from_type_fn early-returned on a `mixed`-typed property,
unlike its var-side sibling (narrow_from_type_fn), which already narrows
mixed/scalar to a concrete type via narrow_to_string/narrow_to_int etc.
`is_string($this->prop)` etc. on an untyped ($var-less docblock) property
narrowed nothing. The false branch stays mixed either way (is_string()
on mixed is false, so the filter is a no-op) — only the true branch gains
narrowing.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
final class Holder {
    public $prop;

    public function narrowsIsStringTrueBranch(): void {
        if (is_string($this->prop)) {
            /** @mir-check $this->prop is string */
            $_ = 1;
        }
    }

    public function narrowsIsIntTrueBranch(): void {
        if (is_int($this->prop)) {
            /** @mir-check $this->prop is int */
            $_ = 1;
        }
    }

    public function falseBranchStaysMixed(): void {
        if (!is_string($this->prop)) {
            /** @mir-check $this->prop is mixed */
            $_ = 1;
        }
    }
}
===expect===
