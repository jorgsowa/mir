===description===
PossiblyInvalidArrayAccess fires when a function parameter has a union type
that includes an int alongside an array.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function process(int|array $data): void {
    echo $data[0];
//       ^^^^^^^^ PossiblyInvalidArrayAccess: Possibly invalid array access: 'int|array' might not support []
}
