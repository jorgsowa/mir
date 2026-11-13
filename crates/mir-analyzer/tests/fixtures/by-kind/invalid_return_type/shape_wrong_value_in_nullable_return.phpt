===description===
A shape with a wrong value is invalid in one branch of a nullable shape return.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @return array{id: int}|null */
function find(bool $found): ?array {
    return $found ? ['id' => 'x'] : null;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'id': "x"}|null' is not compatible with declared 'array{'id': int}|null'
}
