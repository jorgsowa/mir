===description===
function declares @throws for thrown exception - no error
===file===
<?php
/**
 * @throws \RuntimeException
 */
function riskyOperation(): void {
    throw new \RuntimeException('fail');
}
===expect===
===ignore===
