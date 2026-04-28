===file:Entity.php===
<?php
namespace App\Model;
class Entity {}
===file:Service.php===
<?php
use App\Model\Entity;
function wrap(): void {
    $x = new Entity();
}
===expect===
Service.php: UnusedVariable: Variable $x is never read
