===file===
<?php
// Bug: elseif branches were discarded from the post-if merge — variables assigned
// in every branch (if / elseif / else) were incorrectly treated as possibly-undefined.
function foo(int $x): string {
    if ($x > 0) {
        $result = 'positive';
    } elseif ($x < 0) {
        $result = 'negative';
    } else {
        $result = 'zero';
    }
    return $result;
}
===expect===
