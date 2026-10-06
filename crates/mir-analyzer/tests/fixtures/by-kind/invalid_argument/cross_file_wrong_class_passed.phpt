===description===
cross file wrong class passed
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:User.php===
<?php
class User {}
===file:Admin.php===
<?php
class Admin {}
===file:Service.php===
<?php
function createUser(User $u): void { var_dump($u); }
function test(): void {
    createUser(new Admin());
//             ^^^^^^^^^^^ InvalidArgument: Argument $u of createUser() expects 'User', got 'Admin'
}
