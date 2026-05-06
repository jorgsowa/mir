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
===ignore===
BUG: Built-in exception classes need proper Salsa ClassNode registration for inheritance chain resolution to work correctly
