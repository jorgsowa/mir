===description===
a param type referencing a use import with different case resolves to the real FQCN, not the raw written name
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedClass errorLevel="suppress"/>
    <UnusedMethod errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:Lib.php===
<?php
namespace MyApp\Deep;
class Service {}
===file:Consumer.php===
<?php
namespace Client;
use MyApp\Deep;
class C {
    public function take(deep\Service $s): void {}
}
===file:Main.php===
<?php
namespace Client;
$c = new C();
$c->take("not a service");
//       ^^^^^^^^^^^^^^^ InvalidArgument: Argument $s of take() expects 'MyApp\Deep\Service', got '"not a service"'
