===description===
`#[\SensitiveParameter]` takes effect from PHP 8.2, so older targets aren't told
to add it.
===config===
<mir>
  <phpVersion>8.1</phpVersion>
  <issueHandlers>
    <MissingSensitiveParameter errorLevel="warning"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function login(string $user, string $password): bool { return true; }
