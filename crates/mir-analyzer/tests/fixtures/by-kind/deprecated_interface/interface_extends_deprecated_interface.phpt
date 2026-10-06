===description===
An interface that extends a deprecated interface should trigger DeprecatedInterface
===file===
<?php

/**
 * @deprecated Use NewLogger instead
 */
interface OldLogger {
    public function log(string $msg): void;
}

interface ExtendedLogger extends OldLogger {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedInterface: Interface OldLogger is deprecated: Use NewLogger instead
