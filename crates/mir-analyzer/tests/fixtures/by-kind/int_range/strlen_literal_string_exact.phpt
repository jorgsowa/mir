===description===
strlen on a literal string returns the exact byte length as a literal int
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $a = strlen("hello");
    /** @mir-check $a is 5 */
    $_ = $a;

    $b = strlen("");
    /** @mir-check $b is 0 */
    $_ = $b;

    $c = strlen("hello world");
    /** @mir-check $c is 11 */
    $_ = $c;
}
===expect===
