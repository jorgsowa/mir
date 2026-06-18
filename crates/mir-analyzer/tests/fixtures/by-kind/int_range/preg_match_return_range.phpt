===description===
preg_match returns int<0,1>|false; in the truthy branch this narrows to 1.
preg_match_all returns non-negative-int|false.
===config===
suppress=UnusedVariable
===file===
<?php
function test_preg_match_range(string $s): void {
    $r = preg_match('/foo/', $s);
    /** @mir-check $r is int<0, 1>|false */
    $_ = $r;
}

function test_preg_match_all_range(string $s): void {
    $r = preg_match_all('/foo/', $s);
    /** @mir-check $r is non-negative-int|false */
    $_ = $r;
}
===expect===
