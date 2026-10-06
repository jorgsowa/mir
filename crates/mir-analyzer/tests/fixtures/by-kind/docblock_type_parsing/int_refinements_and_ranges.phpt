===description===
Int refinement pseudo-types, `int<min, max>` bounded ranges, and the
`int-mask`/`int-mask-of` flag-set expansion (literal members OR-combined,
including 0 for "no flags").
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function check_positive_int($x) {
    /**
     * @var positive-int $x
     * @mir-check $x is positive-int
     */
    var_dump($x);
}

function check_negative_int($x) {
    /**
     * @var negative-int $x
     * @mir-check $x is negative-int
     */
    var_dump($x);
}

function check_non_negative_int($x) {
    /**
     * @var non-negative-int $x
     * @mir-check $x is non-negative-int
     */
    var_dump($x);
}

function check_literal_int($x) {
    /**
     * @var literal-int $x
     * @mir-check $x is int
     */
    var_dump($x);
}

function check_int_mask_literals($x) {
    /**
     * @var int-mask<1, 2, 4> $x
     * @mir-check $x is 0|1|2|3|4|5|6|7
     */
    var_dump($x);
}

function check_int_mask_keyword($x) {
    /**
     * @var int-mask $x
     * @mir-check $x is int
     */
    var_dump($x);
}

function check_int_range_bounded($x) {
    /**
     * @var int<0, 100> $x
     * @mir-check $x is int<0, 100>
     */
    var_dump($x);
}

function check_int_range_open_min($x) {
    /**
     * @var int<0, max> $x
     * @mir-check $x is int<0, max>
     */
    var_dump($x);
}

function check_int_range_open_max($x) {
    /**
     * @var int<min, 100> $x
     * @mir-check $x is int<min, 100>
     */
    var_dump($x);
}

function check_int_range_unbounded($x) {
    /**
     * @var int<min, max> $x
     * @mir-check $x is int
     */
    var_dump($x);
}

class Flags {
    const A = 1;
    const B = 2;
    const C = 4;

    /**
     * @param int-mask-of<self::*> $mask
     */
    public function set($mask) {
        /** @mir-check $mask is 0|1|2|3|4|5|6|7 */
        var_dump($mask);
    }
}
