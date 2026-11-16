===description===
`password_hash()` options built in a variable first are checked like a literal
array.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <PossiblyInvalidArgument errorLevel="suppress"/>
    <PossiblyFalseArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function store(string $password): void {
    $options = ['cost' => 8];
    password_hash($password, PASSWORD_BCRYPT, $options);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecurePasswordHash: Insecure password hashing: bcrypt cost 8 is below 10
    $salted = ['salt' => 'abcdefghijklmnopqrstuv'];
    password_hash($password, PASSWORD_DEFAULT, $salted);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecurePasswordHash: Insecure password hashing: password_hash() ignores the 'salt' option since PHP 8.0
    $strong = ['cost' => 12];
    password_hash($password, PASSWORD_BCRYPT, $strong);
}
