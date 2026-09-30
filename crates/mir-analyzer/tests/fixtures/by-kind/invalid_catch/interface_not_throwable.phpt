===description===
InvalidCatch fires when the caught type is an interface that does not extend Throwable.
===file===
<?php
interface Loggable {}

try {
    echo "ok";
} catch (Loggable $e) {}
//       ^^^^^^^^ InvalidCatch: Caught type 'Loggable' does not extend Throwable
===expect===
