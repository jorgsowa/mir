===description===
InterfaceInstantiation fires when an interface is instantiated inside a function body.
===file===
<?php
interface Repository {
    public function find(int $id): int;
}

function getRepo(): void {
    new Repository();
}
===expect===
InterfaceInstantiation@7:8-7:18: Cannot instantiate interface Repository
