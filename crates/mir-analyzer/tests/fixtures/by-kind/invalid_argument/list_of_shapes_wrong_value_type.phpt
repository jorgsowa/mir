===description===
A shape nested in a list is checked key by key against the expected shape.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<array{id: int, name: string}> $users */
function saveUsers(array $users): void {}

saveUsers([['id' => 'x', 'name' => 'Ada']]);
//        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $users of saveUsers() expects 'list<array{'id': int, 'name': string}>', got 'array{0: array{'id': "x", 'name': "Ada"}}'
saveUsers([['id' => 1, 'name' => 'Ada'], ['id' => 2, 'name' => 3]]);
//        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $users of saveUsers() expects 'list<array{'id': int, 'name': string}>', got 'array{0: array{'id': 1, 'name': "Ada"}, 1: array{'id': 2, 'name': 3}}'
