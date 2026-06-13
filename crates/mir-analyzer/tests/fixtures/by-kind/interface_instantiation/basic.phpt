===description===
InterfaceInstantiation fires when trying to instantiate an interface.
===file===
<?php
interface Countable {
    public function count(): int;
}

$c = new Countable();
===expect===
InterfaceInstantiation@6:10-6:19: Cannot instantiate interface Countable
