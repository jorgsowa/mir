===description===
Wrong case in namespace prefix segment is reported.
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
$x = new \myapp\service\UserService();
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseClass: Class name 'myapp\service\UserService' has incorrect casing; use 'MyApp\Service\UserService'
