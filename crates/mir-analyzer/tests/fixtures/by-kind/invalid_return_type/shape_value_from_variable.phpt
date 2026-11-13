===description===
A shape value read from a parameter is checked against the declared value type.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @return array{count: int} */
function wrap(string $count): array {
    return ['count' => $count];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'count': string}' is not compatible with declared 'array{'count': int}'
}
