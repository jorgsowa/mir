===description===
new catch use alias in namespaced file no error
===config===
suppress=UnusedVariable,MissingThrowsDocblock,UnusedFunction,InvalidCatch
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
