===description===
Wrong case in namespace prefix segment of a catch clause is reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace MyApp\Exceptions;
class ServiceException extends \RuntimeException {}

namespace Client;
try {
} catch (\myapp\exceptions\ServiceException $e) {
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseClass: Class name 'myapp\exceptions\ServiceException' has incorrect casing; use 'MyApp\Exceptions\ServiceException'
}
===expect===
