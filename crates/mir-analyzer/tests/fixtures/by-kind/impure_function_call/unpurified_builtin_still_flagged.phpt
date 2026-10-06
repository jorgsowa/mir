===description===
Unmarked built-ins remain impure in pure functions.
===file===
<?php
/** @pure */
function persistLogEntry(string $message): void {
    error_log($message);
//  ^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function error_log() in a @pure function
}
