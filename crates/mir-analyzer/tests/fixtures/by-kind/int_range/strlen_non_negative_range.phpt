===description===
strlen() / mb_strlen() are int<0, max>
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(string $s): void {
    $n = strlen($s);
    /** @mir-check $n is int<0, max> */
    $_ = $n;
}
===expect===
