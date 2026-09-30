===description===
Double foreach with inner unused value
===config===
suppress=PossiblyUndefinedVariable,UnusedFunction
===file===
<?php
/**
 * @param non-empty-list<list<int>> $arr
 * @return list<int>
 */
function f(array $arr): array {
    foreach ($arr as $elt) {
        foreach ($elt as $subelt) {}
//                       ^^^^^^^ UnusedForeachValue: Foreach value $subelt is never read
    }
    return $elt;
}

===expect===
