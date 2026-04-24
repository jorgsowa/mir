===file:Entity.php===
<?php
class Entity {
    public function getName(): string { return ""; }
}
===file:Repository.php===
<?php
class Repository {
    public function find(): Entity { return new Entity(); }
}
===file:Service.php===
<?php
function use_repo(Repository $r): void {
    $e = $r->find();
    $e->getName();
    $e->missing();
}
===expect===
Service.php: UndefinedMethod: Method Entity::missing() does not exist
