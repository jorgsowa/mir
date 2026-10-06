===description===
Correct case class name in instanceof is not reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class MyException extends \Exception {}
$e = new MyException();
$result = $e instanceof MyException;
