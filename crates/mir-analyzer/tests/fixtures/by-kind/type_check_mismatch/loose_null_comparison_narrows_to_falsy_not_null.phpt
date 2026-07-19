===description===
`$x == null` narrows to falsy (null/0/0.0/""/[]), not just null — PHP loose-null
equality is also true for those, so collapsing to `TNull` would be unsound.
`!= null` still narrows to non-null exactly like the strict `!== null` form.
Property receivers get the same treatment as plain variables.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
function test_loose_equal_null_narrows_to_falsy(int|null $x): void {
    if ($x == null) {
        /** @mir-check $x is 0|null */
        $_ = $x;
    }
}

function test_loose_not_equal_null_narrows_to_non_null(int|null $x): void {
    if ($x != null) {
        /** @mir-check $x is int */
        $_ = $x;
    }
}

final class Holder {
    public int|null $value;

    public function narrowsLooseEqualNull(): void {
        if ($this->value == null) {
            /** @mir-check $this->value is 0|null */
            $_ = 1;
        }
    }

    public function narrowsLooseNotEqualNull(): void {
        if ($this->value != null) {
            /** @mir-check $this->value is int */
            $_ = 1;
        }
    }
}
===expect===
