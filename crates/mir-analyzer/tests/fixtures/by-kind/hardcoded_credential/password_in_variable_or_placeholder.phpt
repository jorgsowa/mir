===description===
A password held in a variable with one literal value is hard-coded too;
placeholders such as `%env()%`, `${...}`, `{{ }}`, `<...>` and masked
values aren't.
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
function connect(): void {
    $password = 'hunter22';
    new PDO('mysql:host=db', 'app', $password);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HardcodedCredential: Hard-coded credential: the password passed to new PDO() is written in the source
    new PDO('mysql:host=db', 'app', '%env(DB_PASSWORD)%');
    new PDO('mysql:host=db', 'app', '********');
    mysqli_connect('db', 'app', '${DB_PASS}');
    mysqli_connect('db', 'app', '{{ db_password }}');
    mysqli_connect('db', 'app', '<password>');
    pg_connect('host=db user=app password=%env(PG_PASS)%');
}
