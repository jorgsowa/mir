===description===
bounded unbound template parameter does not fabricate receiver type parameter
===config===
suppress=MissingPropertyType
===file:Repo.php===
<?php
class Base {}
class Other {}
/**
 * @template T of Base
 */
class Repo {
    private $id;
    private $item;
    public function __construct(int $id) { $this->id = $id; }
    /** @param T $item */
    public function add($item): void { $this->item = $item; }
}
===file:App.php===
<?php
function app(): void {
    $r = new Repo(5);
    $r->add(new Other());
}
===expect===
