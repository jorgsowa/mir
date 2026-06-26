===description===
InterfaceInstantiation fires when an interface is instantiated inside a class method body.
===config===
suppress=UnusedVariable
===file===
<?php
interface Storage {
    public function store(string $key, mixed $value): void;
}

class Cache {
    public function clear(): void {
        $s = new Storage();
    }
}
===expect===
InterfaceInstantiation@8:17-8:24: Cannot instantiate interface Storage
