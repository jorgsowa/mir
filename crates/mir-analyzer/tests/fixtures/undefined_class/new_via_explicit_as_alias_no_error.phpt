===description===
new via explicit as alias no error
===file:Entity.php===
<?php
namespace App\Model;
class Entity {}
===file:Service.php===
<?php
use App\Model\Entity as E;
function wrap(): void {
    $x = new E();
}
===expect===
Service.php: UnusedVariable@4:4: Variable $x is never read
