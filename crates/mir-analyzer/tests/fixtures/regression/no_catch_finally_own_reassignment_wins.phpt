===description===
Companion to no_catch_finally_does_not_leak_pessimistic_state: no catch
clause, but `finally` itself unconditionally reassigns $str. The
post-statement state must reflect finally's own guaranteed effect (a
plain string) rather than json_encode()'s pre-finally type (string|false)
or the exception-can-happen-anywhere merge — so passing $str to a
strict `string` parameter afterward must not flag PossiblyInvalidArgument.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function needsString(string $s): void {}

function encode(mixed $value): void {
    try {
        $str = json_encode($value);
    } finally {
        $str = 'x';
    }
    needsString($str);
}
===expect===
