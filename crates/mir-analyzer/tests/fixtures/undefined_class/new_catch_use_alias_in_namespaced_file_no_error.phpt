===description===
new catch use alias in namespaced file no error
===file:Entity.php===
<?php
namespace App\Model;
class Entity {}
===file:Service.php===
<?php
namespace App\Service;
use App\Model\Entity;
function wrap(): void {
    $x = new Entity();
    try {
        throw new \Exception();
    } catch (Entity $e) {}
}
===expect===
Service.php: UnusedVariable@5:4: Variable $x is never read
Service.php: MissingThrowsDocblock@7:8: Exception Exception is thrown but not declared in @throws
Service.php: UnusedVariable@8:12: Variable $e is never read
