===description===
new via explicit as alias no error
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
