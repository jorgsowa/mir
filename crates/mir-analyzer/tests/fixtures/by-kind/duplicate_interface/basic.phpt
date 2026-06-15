===description===
DuplicateInterface fires when the same interface is declared twice.
===file===
<?php
interface Logger {
    public function log(string $msg): void;
}

interface Logger {
    public function write(string $msg): void;
}
===expect===
DuplicateInterface@6:0-8:1: Interface Logger has already been defined
