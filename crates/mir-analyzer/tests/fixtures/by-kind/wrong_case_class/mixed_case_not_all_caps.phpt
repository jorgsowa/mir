===description===
Mixed case variants (not all-caps) are also detected.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class HttpClient {}
$c = new httpclient();
//       ^^^^^^^^^^ WrongCaseClass: Class name 'httpclient' has incorrect casing; use 'HttpClient'
$d = new HttpCLIENT();
//       ^^^^^^^^^^ WrongCaseClass: Class name 'HttpCLIENT' has incorrect casing; use 'HttpClient'
