===description===
The whole `is_*`/`ctype_*`/`array_is_list`/`method_exists`/`property_exists`
type-check dispatch (`narrow_from_type_fn`) previously only ever narrowed a
plain-variable receiver — unlike the analogous `instanceof`/null/literal-match
arms elsewhere in narrowing.rs, which all already have a property-access
fallback. `is_string($this->prop)` etc. narrowed nothing.
===config===
suppress=UnusedVariable,MissingConstructor,MixedArgument,MissingPropertyType
===file===
<?php
final class Holder {
    /** @var int|string */
    public $scalar;

    public function narrowsIsStringTrueBranch(): void {
        if (is_string($this->scalar)) {
            /** @mir-check $this->scalar is string */
            $_ = 1;
        }
    }

    public function narrowsIsIntTrueBranch(): void {
        if (is_int($this->scalar)) {
            /** @mir-check $this->scalar is int */
            $_ = 1;
        }
    }

    public function narrowsFalseBranch(): void {
        if (!is_string($this->scalar)) {
            /** @mir-check $this->scalar is int */
            $_ = 1;
        }
    }

    /** @var array<int, string>|string */
    public $shape;

    public function narrowsIsArray(): void {
        if (is_array($this->shape)) {
            /** @mir-check $this->shape is array<int, string> */
            $_ = 1;
        }
    }

    /** @var list<int>|array{a: string} */
    public $listy;

    public function narrowsArrayIsList(): void {
        if (array_is_list($this->listy)) {
            /** @mir-check $this->listy is list<int> */
            $_ = 1;
        }
    }

    /** @var string */
    public $str;

    public function narrowsCtypeDigit(): void {
        if (ctype_digit($this->str)) {
            /** @mir-check $this->str is non-empty-string */
            $_ = 1;
        }
    }

    /** @var object|string */
    public $methodTarget;

    public function narrowsMethodExists(): void {
        if (method_exists($this->methodTarget, 'foo')) {
            /** @mir-check $this->methodTarget is object|string */
            $_ = 1;
        }
    }
}
===expect===
