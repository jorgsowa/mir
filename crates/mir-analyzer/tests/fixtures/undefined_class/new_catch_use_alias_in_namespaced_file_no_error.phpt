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
Service.php: UnusedVariable: Variable $x is never read
Service.php: UnusedVariable: Variable $e is never read
