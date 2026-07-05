===description===
Loop conditions (while/for/do-while) must run the same docblock-contradiction
and redundant-condition checks as `if` — an always-false condition must not
silently let the loop body/continuation go unchecked.
===config===
suppress=UnusedParam
===file===
<?php
/** @param int<5, max> $n */
function test_while(int $n): void {
    while ($n < 4) {
        echo "never";
    }
}

/** @param int<5, max> $n */
function test_for(int $n): void {
    for ($i = 0; $n < 4; $i++) {
        echo "never";
    }
}

/** @param int<5, max> $n */
function test_dowhile(int $n): void {
    do {
        echo "runs once";
    } while ($n < 4);
}
===expect===
DocblockTypeContradiction@4:11-4:17: Type 'int<5, max>' makes '$n < 4' impossible — this can never hold
RedundantCondition@4:11-4:17: Condition is always true/false for type 'bool'
UnreachableCode@5:8-5:21: Unreachable code detected
DocblockTypeContradiction@11:17-11:23: Type 'int<5, max>' makes '$n < 4' impossible — this can never hold
RedundantCondition@11:17-11:23: Condition is always true/false for type 'bool'
UnreachableCode@12:8-12:21: Unreachable code detected
DocblockTypeContradiction@20:13-20:19: Type 'int<5, max>' makes '$n < 4' impossible — this can never hold
