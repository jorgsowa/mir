===description===
`properties-of<Cls>` is a shape of the class's instance properties, so a wrongly typed or missing entry is rejected.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Account {
    public string $email = '';
    public int $balance = 0;
    public static int $instances = 0;
}

/** @param properties-of<Account> $row */
function hydrate(array $row): void {}

hydrate(['email' => 'a@example.com', 'balance' => 10]);
hydrate(['email' => 'a@example.com', 'balance' => 'ten']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $row of hydrate() expects 'array{'email': string, 'balance': int}', got 'array{'email': "a@example.com", 'balance': "ten"}'
hydrate(['email' => 'a@example.com']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $row of hydrate() expects 'array{'email': string, 'balance': int}', got 'array{'email': "a@example.com"}'
hydrate('a@example.com');
//      ^^^^^^^^^^^^^^^ InvalidArgument: Argument $row of hydrate() expects 'array{'email': string, 'balance': int}', got '"a@example.com"'
