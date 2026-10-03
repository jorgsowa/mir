===description===
Integer, negative-integer, float, negative-float, and single-/double-quoted
string literal types — including that the two quote styles produce the same
literal-string atom.
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
function check_int_literal($x) {
    /**
     * @var 42 $x
     * @mir-check $x is 42
     */
    var_dump($x);
}

function check_negative_int_literal($x) {
    /**
     * @var -42 $x
     * @mir-check $x is -42
     */
    var_dump($x);
}

function check_zero_literal($x) {
    /**
     * @var 0 $x
     * @mir-check $x is 0
     */
    var_dump($x);
}

function check_float_literal($x) {
    /**
     * @var 3.14 $x
     * @mir-check $x is 3.14
     */
    var_dump($x);
}

function check_negative_float_literal($x) {
    /**
     * @var -0.5 $x
     * @mir-check $x is -0.5
     */
    var_dump($x);
}

function check_single_quoted_string_literal($x) {
    /**
     * @var 'hello' $x
     * @mir-check $x is 'hello'
     */
    var_dump($x);
}

function check_double_quoted_string_literal($x) {
    /**
     * @var "world" $x
     * @mir-check $x is "world"
     */
    var_dump($x);
}

function check_mixed_quote_equivalence($x) {
    /**
     * @var 'same' $x
     * @mir-check $x is "same"
     */
    var_dump($x);
}
===expect===
