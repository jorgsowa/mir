===description===
An assignment inside `finally` propagates to code after the try statement
— finally always runs last, so its reassignment is authoritative
===file===
<?php
function f(mixed $x): void {
    try {
        $x = 1;
    } finally {
        $x = "hello";
    }
    /** @mir-check $x is string */
    echo $x;
}
===expect===
