===description===
Exit is like throw
===file===
<?php
/**
 * @param 1|2|3 $i
 */
function foo(int $i): void {
    $a = match ($i) {
        1 => exit(),
        2, 3 => $i,
    };
    $a === "aaa";
//  ^^^^^^^^^^^^ DocblockTypeContradiction: Type '2|3' makes '$a === "aaa"' impossible — this can never hold
//  ^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '2|3' and '"aaa"' is always false — these types can never be identical
}
