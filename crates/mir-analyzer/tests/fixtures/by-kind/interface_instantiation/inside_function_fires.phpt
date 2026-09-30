===description===
InterfaceInstantiation fires when an interface is instantiated inside a function body.
===file===
<?php
interface Repository {
    public function find(int $id): int;
}

function getRepo(): void {
    new Repository();
//      ^^^^^^^^^^ InterfaceInstantiation: Cannot instantiate interface Repository
}
===expect===
