===description===
Refined scalar spellings without an atom of their own parse to the closest type.
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
function check_non_zero_int($x) {
    /**
     * @var non-zero-int $x
     * @mir-check $x is negative-int|positive-int
     */
    var_dump($x);
}

function check_non_positive_int($x) {
    /**
     * @var non-positive-int $x
     * @mir-check $x is int<min, 0>
     */
    var_dump($x);
}

function check_int_range($x) {
    /**
     * @var int-range<0, 255> $x
     * @mir-check $x is int<0, 255>
     */
    var_dump($x);
}

function check_number($x) {
    /**
     * @var number $x
     * @mir-check $x is int|float
     */
    var_dump($x);
}

function check_decimal_int_string($x) {
    /**
     * @var decimal-int-string $x
     * @mir-check $x is numeric-string
     */
    var_dump($x);
}

function check_non_decimal_int_string($x) {
    /**
     * @var non-decimal-int-string $x
     * @mir-check $x is string
     */
    var_dump($x);
}

function check_non_empty_literal_string($x) {
    /**
     * @var non-empty-literal-string $x
     * @mir-check $x is non-empty-literal-string
     */
    var_dump($x);
}

function check_non_empty_scalar($x) {
    /**
     * @var non-empty-scalar $x
     * @mir-check $x is scalar
     */
    var_dump($x);
}

function check_non_empty_mixed($x) {
    /**
     * @var non-empty-mixed $x
     * @mir-check $x is mixed
     */
    var_dump($x);
}

function check_non_null_mixed($x) {
    /**
     * @var non-null-mixed $x
     * @mir-check $x is mixed
     */
    var_dump($x);
}
