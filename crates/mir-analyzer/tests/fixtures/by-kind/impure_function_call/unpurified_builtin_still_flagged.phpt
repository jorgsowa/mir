===description===
An unpurified stub builtin (error_log, no @pure/#[Pure]) called inside a
@pure function IS still flagged as an impure call — proving #[Pure] attribute
detection is what enables the exemption.
===file===
<?php
/** @pure */
function persistLogEntry(string $message): void {
    error_log($message);
}
===expect===
ImpureFunctionCall@4:4-4:23: Calling impure function error_log() in a @pure function
