===description===
A single closure capturing two undefined by-ref variables, one matching
its own self-referential idiom (`&$tally`, same name the closure literal
is assigned to) and one that doesn't (`&$total`, a genuine out-param).
Confirms the fix decides per-capture, inside one `use()` list, rather than
seeding every capture on a self-referential closure as callable (or every
capture on any closure as mixed).
===config===
suppress=MixedAssignment
===file===
<?php
function run(): void {
    $tally = function () use (&$tally, &$total): void {
        $total = ($total ?? 0) + 1;
        if ($total < 3) {
            $tally();
        }
    };
    $tally();
    if ($total) {
        echo $total;
    }
}
===expect===
