===description===
Regression guard: code after a loop is not itself "inside the loop" — a
push onto a freshly-empty array declared after the loop body still grows a
precise shape instead of inheriting the loop's forced generalization.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $n): void {
    for ($i = 0; $i < $n; $i++) {
        $inside = [];
        $inside[] = $i;
    }
    $after = [];
    $after[] = 1;
    /** @mir-check $after is array{0: 1} */
    $_ = $after;
}
===expect===
