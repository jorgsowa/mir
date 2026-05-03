===description===
new via use alias cross file no error
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
Service.php: UnusedVariable@4:4: Variable $x is never read
===ignore===
TODO
