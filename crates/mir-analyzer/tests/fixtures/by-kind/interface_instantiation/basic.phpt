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
//       ^^^^^^^^^ InterfaceInstantiation: Cannot instantiate interface Countable
===expect===
