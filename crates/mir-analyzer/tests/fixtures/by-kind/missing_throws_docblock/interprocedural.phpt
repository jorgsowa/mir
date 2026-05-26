===description===
function calls function that declares @throws without declaring it itself
===file===
<?php
/**
 * @throws \Exception
 */
function riskyOperation(): void {
    throw new \Exception('fail');
}

function callerNoThrows(): void {
    riskyOperation();
}
===expect===
MissingThrowsDocblock@10:5: Exception Exception is thrown but not declared in @throws
