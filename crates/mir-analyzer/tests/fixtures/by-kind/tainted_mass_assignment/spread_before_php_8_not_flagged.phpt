===description===
Before PHP 8.0 a spread array can't carry named arguments.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function createUser(string $name, string $email = ''): void {}

function viaFunction(): void {
    createUser(...$_POST);
}
