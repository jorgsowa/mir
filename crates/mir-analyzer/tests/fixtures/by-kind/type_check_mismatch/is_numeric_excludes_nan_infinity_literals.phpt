===description===
is_numeric() true-branch excludes the 'NAN' literal from a 'NAN'|'42' union
— 'NAN' is not a PHP numeric string, unlike '42'. False-branch keeps it.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param 'NAN'|'42' $s */
function test_true_branch(string $s): void {
    if (is_numeric($s)) {
        /** @mir-check $s is '42' */
        $_ = $s;
    }
}
/** @param 'NAN'|'42' $s */
function test_false_branch(string $s): void {
    if (!is_numeric($s)) {
        /** @mir-check $s is 'NAN' */
        $_ = $s;
    }
}
===expect===
