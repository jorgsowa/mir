===description===
Partial namespace import with wrong inner namespace segment reports the fully resolved path.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace MyApp\Deep\Service;
class UserService {}

namespace Client;
use MyApp\Deep;
$x = new Deep\service\UserService();
//       ^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseClass: Class name 'MyApp\Deep\service\UserService' has incorrect casing; use 'MyApp\Deep\Service\UserService'
===expect===
