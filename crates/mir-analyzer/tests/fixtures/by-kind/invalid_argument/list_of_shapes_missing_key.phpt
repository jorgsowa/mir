===description===
A shape nested in a list must have every required key of the expected shape.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<array{id: int, name: string, email?: string}> $users */
function saveUsers(array $users): void {}

saveUsers([['id' => 1]]);
//        ^^^^^^^^^^^^^ InvalidArgument: Argument $users of saveUsers() expects 'list<array{'id': int, 'name': string, 'email'?: string}>', got 'array{0: array{'id': 1}}'
saveUsers([['id' => 1, 'name' => 'Ada']]);
