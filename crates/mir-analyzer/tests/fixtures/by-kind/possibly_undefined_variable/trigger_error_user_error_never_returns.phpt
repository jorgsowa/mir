===description===
`trigger_error()` with `E_USER_ERROR` halts the script; other levels return.
===file===
<?php
function make(): object { return new stdClass(); }

function userError(): object {
    try {
        $o = make();
    } catch (Throwable $e) {
        trigger_error($e->getMessage(), E_USER_ERROR);
    }
    return $o;
}

function userWarning(): object {
    try {
        $o = make();
    } catch (Throwable $e) {
        trigger_error($e->getMessage(), E_USER_WARNING);
    }
    return $o;
//         ^^ PossiblyUndefinedVariable: Variable $o might not be defined
}
