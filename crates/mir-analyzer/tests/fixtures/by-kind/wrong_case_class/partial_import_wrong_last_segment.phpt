===description===
Partial namespace import followed by wrong-case last segment is reported.
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
$x = new Service\userservice();
//       ^^^^^^^^^^^^^^^^^^^ WrongCaseClass: Class name 'userservice' has incorrect casing; use 'UserService'
===expect===
