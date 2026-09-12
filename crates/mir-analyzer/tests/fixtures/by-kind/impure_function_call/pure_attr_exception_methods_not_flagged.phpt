===description===
Pure-marked methods are allowed in pure functions.
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
