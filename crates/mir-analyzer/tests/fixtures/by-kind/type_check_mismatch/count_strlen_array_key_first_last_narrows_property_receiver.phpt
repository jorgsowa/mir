===description===
count()/strlen()/array_key_first()/array_key_last() narrowing on a
property receiver (`$this->prop`), the property-access counterpart of the
already-existing plain-variable narrowing for the same builtins.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
final class Holder {
    /** @var list<int>|non-empty-array<string, int> */
    public $arr;
    /** @var non-empty-string|numeric-string */
    public $str;

    public function countEquality(): void {
        if (count($this->arr) === 0) {
            /** @mir-check $this->arr is list<int> */
            $_ = 1;
        }
    }

    public function countRelational(): void {
        if (count($this->arr) > 0) {
            /** @mir-check $this->arr is non-empty-list<int>|non-empty-array<string, int> */
            $_ = 1;
        }
    }

    public function reversedCountRelational(): void {
        if (0 === count($this->arr)) {
            /** @mir-check $this->arr is list<int> */
            $_ = 1;
        }
    }

    // `numeric-string` can never be "" (is_numeric('') is false in PHP), so
    // this branch is unreachable for either atom; the type stays unchanged
    // rather than asserting an impossible narrower type.
    public function strlenEquality(): void {
        if (strlen($this->str) === 0) {
            /** @mir-check $this->str is non-empty-string|numeric-string */
            $_ = 1;
        }
    }

    public function strlenRelational(): void {
        if (strlen($this->str) < 1) {
            /** @mir-check $this->str is non-empty-string|numeric-string */
            $_ = 1;
        }
    }

    public function arrayKeyFirstNull(): void {
        if (array_key_first($this->arr) === null) {
            /** @mir-check $this->arr is list<int> */
            $_ = 1;
        }
    }

    public function arrayKeyLastNullReversed(): void {
        if (null === array_key_last($this->arr)) {
            /** @mir-check $this->arr is list<int> */
            $_ = 1;
        }
    }
}
===expect===
