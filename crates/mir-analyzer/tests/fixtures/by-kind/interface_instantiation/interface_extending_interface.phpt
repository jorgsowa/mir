===description===
InterfaceInstantiation fires for an interface that extends another interface.
===config===
suppress=UnusedVariable
===file===
<?php
interface Loggable {
    public function log(): void;
}

interface StructuredLoggable extends Loggable {
    public function structuredLog(): array;
}

$l = new StructuredLoggable();
===expect===
InterfaceInstantiation@10:9-10:27: Cannot instantiate interface StructuredLoggable
