===description===
A tuple returned for a tuple shape is checked value by value.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @return array{0: int, 1: int} */
function pair(): array {
    return ['a', 'b'];
//  ^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{0: "a", 1: "b"}' is not compatible with declared 'array{0: int, 1: int}'
}
