===description===
date(), gmdate(), and date_format() always return non-empty-string.
===config===
suppress=UnusedVariable
===file===
<?php
function test_date(): void {
    $r = date('Y-m-d');
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_gmdate(): void {
    $r = gmdate('H:i:s');
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_date_format(): void {
    $dt = new DateTimeImmutable();
    $r = date_format($dt, 'Y-m-d H:i:s');
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}
===expect===
