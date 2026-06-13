===description===
DeprecatedInterface fires when a class implements a deprecated interface.
===file===
<?php
/** @deprecated use NewLogger instead */
interface OldLogger {
    public function log(string $msg): void;
}

class FileLogger implements OldLogger {
    public function log(string $msg): void {}
}
===expect===
DeprecatedInterface@7:0-7:39: Interface OldLogger is deprecated: use NewLogger instead
