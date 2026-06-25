===description===
InvalidCatch fires when the caught type is an interface that does not extend Throwable.
===file===
<?php
interface Loggable {}

try {
    echo "ok";
} catch (Loggable $e) {}
===expect===
InvalidCatch@6:9-6:17: Caught type 'Loggable' does not extend Throwable
