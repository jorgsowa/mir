===description===
DuplicateInterface fires when the same interface is declared twice.
===file===
<?php
interface Logger {
    public function log(string $msg): void;
}

interface Logger {
//<^ +2:1 DuplicateInterface: Interface Logger has already been defined
    public function write(string $msg): void;
}
