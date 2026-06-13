===description===
InterfaceInstantiation does NOT fire when instantiating a concrete class.
===config===
suppress=UnusedVariable
===file===
<?php
class FileLogger implements \Countable {
    public function count(): int { return 0; }
}

$l = new FileLogger();
===expect===
