===description===
!is_float()/!is_double() false branch also excludes TIntegralFloat (e.g. floor()).
===config===
suppress=UnusedVariable,UnusedParam,MixedArgument,MissingReturnType
===file===
<?php
function test_true_branch(bool $cond, string $s) {
    $x = $cond ? floor(3.5) : $s;
    if (is_float($x)) {
        /** @mir-check $x is float */
        $_ = $x;
    }
}

function test_false_branch(bool $cond, string $s) {
    $x = $cond ? floor(3.5) : $s;
    if (!is_float($x)) {
        /** @mir-check $x is string */
        $_ = $x;
    }
}

function test_double_false_branch(bool $cond, string $s) {
    $x = $cond ? floor(3.5) : $s;
    if (!is_double($x)) {
        /** @mir-check $x is string */
        $_ = $x;
    }
}
===expect===
