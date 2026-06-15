===description===
InterfaceInstantiation fires when trying to instantiate an interface.
===config===
suppress=UnusedVariable
===file===
<?php
interface Countable {
    public function count(): int;
}

$c = new Countable();
===expect===
InterfaceInstantiation@6:9-6:18: Cannot instantiate interface Countable
