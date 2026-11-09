===description===
A literal assignment can't throw, so when it opens the `try` the catch block sees the variable defined.
===file===
<?php
function t(string $s): void { if ($s === '') { throw new RuntimeException(); } }
function f(): void {
    try {
        $a = 'x';
        t($a);
    } catch (RuntimeException $e) {
        echo $a;
    }
}
