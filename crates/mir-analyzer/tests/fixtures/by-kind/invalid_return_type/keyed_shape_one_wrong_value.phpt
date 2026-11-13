===description===
One value of the wrong type makes a returned shape invalid; the matching shape is accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @return array{id: int, name: string} */
function bad(): array {
    return ['id' => 1, 'name' => 2];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'id': 1, 'name': 2}' is not compatible with declared 'array{'id': int, 'name': string}'
}

/** @return array{id: int, name: string} */
function good(): array {
    return ['id' => 1, 'name' => 'x'];
}
