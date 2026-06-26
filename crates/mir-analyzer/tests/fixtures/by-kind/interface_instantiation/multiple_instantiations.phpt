===description===
InterfaceInstantiation fires independently for each interface instantiation in the same file.
===config===
suppress=UnusedVariable
===file===
<?php
interface Readable {
    public function read(): string;
}

interface Writable {
    public function write(string $data): void;
}

$r = new Readable();
$w = new Writable();
===expect===
InterfaceInstantiation@10:9-10:17: Cannot instantiate interface Readable
InterfaceInstantiation@11:9-11:17: Cannot instantiate interface Writable
