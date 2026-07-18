===description===
in_array()'s property-access counterpart: `in_array($this->prop, [...])`
narrows the property receiver the same way a plain-variable needle already
does, for both the true branch (intersect with the haystack) and the false
branch (remove matched literals from a finite literal-union property type).
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
final class Request {
    /** @var string */
    public string $mode;

    /** @var "a"|"b"|"c"|"d" */
    public string $status;

    public function trueBranchNarrows(): void {
        if (in_array($this->mode, ['read', 'write', 'append'])) {
            /** @mir-check $this->mode is "read"|"write"|"append" */
            $_ = 1;
        }
    }

    public function falseBranchRemovesLiterals(): void {
        if (!in_array($this->status, ['a', 'b'])) {
            /** @mir-check $this->status is "c"|"d" */
            $_ = 1;
        }
    }
}
===expect===
