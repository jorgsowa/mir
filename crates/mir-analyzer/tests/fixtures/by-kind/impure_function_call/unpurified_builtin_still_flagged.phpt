===description===
Unmarked built-ins remain impure in pure functions.
===file===
<?php
/** @pure */
function persistLogEntry(string $message): void {
    error_log($message);
}
===expect===
ImpureFunctionCall@4:4-4:23: Calling impure function error_log() in a @pure function
