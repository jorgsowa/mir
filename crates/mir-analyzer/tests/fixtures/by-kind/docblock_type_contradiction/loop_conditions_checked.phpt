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
//         ^^^^^^ DocblockTypeContradiction: Type 'int<5, max>' makes '$n < 4' impossible — this can never hold
//         ^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        echo "never";
//      ^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
    }
}

/** @param int<5, max> $n */
function test_for(int $n): void {
    for ($i = 0; $n < 4; $i++) {
//               ^^^^^^ DocblockTypeContradiction: Type 'int<5, max>' makes '$n < 4' impossible — this can never hold
//               ^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        echo "never";
//      ^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
    }
}

/** @param int<5, max> $n */
function test_dowhile(int $n): void {
    do {
        echo "runs once";
    } while ($n < 4);
//           ^^^^^^ DocblockTypeContradiction: Type 'int<5, max>' makes '$n < 4' impossible — this can never hold
}
===expect===
