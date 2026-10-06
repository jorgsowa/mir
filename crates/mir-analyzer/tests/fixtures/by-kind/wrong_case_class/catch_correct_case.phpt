===description===
Correct case class name in catch clause is not reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class AppException extends \RuntimeException {}
try {
    throw new AppException("err");
} catch (AppException $e) {
}
