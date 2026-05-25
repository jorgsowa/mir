===description===
new via explicit as alias no error
===config===
suppress=UnusedVariable,UnusedFunction
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
