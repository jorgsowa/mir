===description===
function throws subclass of declared @throws exception - no error
===file===
<?php
/**
 * @throws \Exception
 */
function riskyOperation(): void {
    throw new \LogicException('fail');
}
===expect===
