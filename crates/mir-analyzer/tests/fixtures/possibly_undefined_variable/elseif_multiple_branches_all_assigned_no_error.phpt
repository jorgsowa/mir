===file===
<?php
// Bug: with multiple elseif branches, only the last one survived the merge loop —
// variables assigned in earlier elseif branches were dropped.
function classify(int $x): string {
    if ($x < 0) {
        $label = 'negative';
    } elseif ($x === 0) {
        $label = 'zero';
    } elseif ($x < 10) {
        $label = 'small';
    } else {
        $label = 'large';
    }
    return $label;
}
===expect===
