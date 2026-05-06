===description===
function calls function that declares @throws without declaring it itself
===file===
<?php
/**
 * @throws \RuntimeException
 */
function riskyOperation(): void {
    throw new \RuntimeException('fail');
}

function callerNoThrows(): void {
    riskyOperation();
}
===expect===
===ignore===
TODO: Inter-procedural throw detection depends on proper stub class loading
