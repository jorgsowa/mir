===description===
`is_int($this->prop) || is_string($this->prop) || is_float($this->prop)`
(a 3-way+ OR-chain, parsed left-associatively as `(a || b) || c`) narrows
the property — `single_leaf_disjunct_prop` didn't recurse into a nested
`||`, unlike its var-side sibling `single_leaf_disjunct_var`.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
final class Holder {
    /** @var int|string|float|bool */
    public $prop;

    public function narrows(): void {
        if (is_int($this->prop) || is_string($this->prop) || is_float($this->prop)) {
            /** @mir-check $this->prop is int|string|float */
            $_ = 1;
        }
    }
}

final class TwoHolders {
    /** @var int|string */
    public $a;
    /** @var int|string */
    public $b;

    public function differentPropsDoNotMerge(): void {
        if (is_int($this->a) || is_string($this->b) || is_int($this->a)) {
            // Disjuncts reference two different properties — no single
            // property can be attributed the union, so nothing narrows.
            /** @mir-check $this->a is int|string */
            $_ = 1;
        }
    }
}
===expect===
