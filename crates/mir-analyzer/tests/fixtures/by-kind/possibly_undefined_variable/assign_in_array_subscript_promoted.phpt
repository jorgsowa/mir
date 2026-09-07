===description===
Assignment nested inside an array subscript within a `&&` condition should be promoted
to definitely-assigned in the true-branch. Pattern: `$arr[$n = count($arr) - 1]`.
`promote_assignment_effects` previously didn't recurse into `ArrayAccess` nodes.
===config===
php_version=8.1
suppress=RedundantCast
===file===
<?php
declare(strict_types=1);

function binary_search(array $table, int $target): bool {
    if ($target >= $table[0] && $target <= $table[$n = count($table) - 1]) {
        $lo = 0;
        while ($n >= $lo) {   // $n is definitely assigned here
            $mid = (int)(($lo + $n) / 2);
            if ($target > $table[$mid]) {
                $lo = $mid + 1;
            } elseif ($target < $table[$mid]) {
                $n = $mid - 1;
            } else {
                return true;
            }
        }
    }
    return false;
}
===expect===
