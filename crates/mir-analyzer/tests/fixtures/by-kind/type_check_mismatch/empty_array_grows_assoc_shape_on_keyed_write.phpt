===description===
A keyed write (`$counts['total'] = …`) onto a proven-empty plain
`array<string, int>` grows the closed shape by that one key, the same way
push notation grows a list shape.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string, int> $counts */
function test(array $counts): void {
    if ($counts === []) {
        $counts['total'] = 5;
        /** @mir-check $counts is array{'total': 5} */
        $_ = $counts;
    }
}
===expect===
