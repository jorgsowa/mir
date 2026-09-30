===description===
Pure-marked methods are allowed in pure functions.
===file===
<?php
/** @pure */
function codeOf(string $message): int {
    return new Exception($message)->getCode();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
}

/** @pure */
function fileOf(Exception $e): string {
    return $e->getFile();
}
===expect===
