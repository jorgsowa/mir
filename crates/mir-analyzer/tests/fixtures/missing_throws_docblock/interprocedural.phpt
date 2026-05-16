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
MissingThrowsDocblock@10:4: Exception Exception is thrown but not declared in @throws
