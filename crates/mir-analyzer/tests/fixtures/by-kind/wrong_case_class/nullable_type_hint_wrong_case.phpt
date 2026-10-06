===description===
Wrong case class name in nullable type hint is reported.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class User {}
function find(int $id): ?user { return null; }
//                       ^^^^ WrongCaseClass: Class name 'user' has incorrect casing; use 'User'
