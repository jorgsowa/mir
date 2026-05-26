===description===
function calls function that declares @throws and also declares it - no error
===file===
<?php
/**
 * @throws \RuntimeException
 */
function riskyOperation(): void {
    throw new \RuntimeException('fail');
}

/**
 * @throws \RuntimeException
 */
function callerWithThrows(): void {
    riskyOperation();
}
===expect===
