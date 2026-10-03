===description===
Correct partial namespace import usage is not reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace MyApp\Service;
class UserService {}

namespace Client;
use MyApp\Service;
$x = new Service\UserService();
===expect===
