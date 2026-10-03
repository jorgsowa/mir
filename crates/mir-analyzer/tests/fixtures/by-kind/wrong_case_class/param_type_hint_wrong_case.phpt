===description===
Wrong case class name in parameter type hint is reported.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Request {}
function handle(request $r): void {}
//              ^^^^^^^ WrongCaseClass: Class name 'request' has incorrect casing; use 'Request'
===expect===
