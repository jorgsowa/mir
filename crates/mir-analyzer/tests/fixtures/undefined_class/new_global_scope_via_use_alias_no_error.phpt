===file:Entity.php===
<?php
namespace App\Model;
class Entity {}
===file:Service.php===
<?php
use App\Model\Entity;
$x = new Entity();
===expect===
