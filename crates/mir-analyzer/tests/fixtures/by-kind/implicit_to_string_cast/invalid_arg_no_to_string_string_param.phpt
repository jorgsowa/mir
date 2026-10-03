===description===
InvalidArgument when object has neither __toString nor Stringable and is passed to a string param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Opaque {}

function render(string $s): void {}

render(new Opaque());
//     ^^^^^^^^^^^^ InvalidArgument: Argument $s of render() expects 'string', got 'Opaque'
===expect===
