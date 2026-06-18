===description===
positive-int >= 5 is NOT always true (can be 1-4), so no RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test(int $n): void {
    if ($n >= 5) {
        echo "not always";
    }
}
===expect===
