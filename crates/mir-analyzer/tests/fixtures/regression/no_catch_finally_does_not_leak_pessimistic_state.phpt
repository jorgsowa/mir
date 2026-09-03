===description===
Reproduces P15: a `try`/`finally` with NO catch clause leaked finally's
pessimistic mid-try merged state (used to analyze finally's own body,
since an exception could occur at any point in try) onto the
post-statement flow state. Since there is no catch clause, an exception
thrown in try propagates past the whole statement after finally runs —
it never reaches code after the statement — so reaching that code means
try completed and $str was assigned.
===config===
suppress=UnusedVariable
===file===
<?php
function encode(mixed $value): void {
    try {
        $str = json_encode($value);
    } finally {
        // cleanup, does not touch $str
    }
    assert($str !== false);
}
===expect===
