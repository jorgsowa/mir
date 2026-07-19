===description===
`extract_haystack_type` only matched an array literal or a plain-variable
haystack argument, not a property-access one -- `in_array($x,
$this->allowedValues)` never narrowed `$x` even though
`$this->allowedValues` carries a known literal-array shape, unlike the
needle side which already had a property-receiver counterpart.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
final class Validator {
    /** @var array{0: "a", 1: "b", 2: "c"} */
    public array $allowedValues = ['a', 'b', 'c'];

    /** @param "a"|"b"|"c"|"d" $mode */
    public function trueBranchNarrows($mode): void {
        if (in_array($mode, $this->allowedValues, true)) {
            /** @mir-check $mode is "a"|"b"|"c" */
            $_ = 1;
        }
    }

    /** @param "a"|"b"|"c"|"d" $mode */
    public function falseBranchRemovesLiterals($mode): void {
        if (!in_array($mode, $this->allowedValues, true)) {
            /** @mir-check $mode is "d" */
            $_ = 1;
        }
    }
}
===expect===
