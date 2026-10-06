===description===
psr4 new via use as alias no error
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Model/Entity.php===
<?php
namespace App\Model;
class Entity {}
===file:Service.php===
<?php
use App\Model\Entity as E;
function wrap(): void {
    $x = new E();
}
