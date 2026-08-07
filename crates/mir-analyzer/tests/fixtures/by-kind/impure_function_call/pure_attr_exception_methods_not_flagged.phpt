===description===
#[Pure]-marked stub methods (e.g. Exception::getCode) called inside a @pure
function are not flagged as impure calls.
===file===
<?php
/** @pure */
function codeOf(string $message): int {
    return new Exception($message)->getCode();
}

/** @pure */
function fileOf(Exception $e): string {
    return $e->getFile();
}
===expect===
MixedReturnStatement@4:4-4:46: Cannot return a mixed type from function with declared return type 'int'
