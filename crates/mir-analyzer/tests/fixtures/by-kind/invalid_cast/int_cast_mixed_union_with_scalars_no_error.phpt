===description===
(int) cast on a union that includes array but also scalar-safe atoms (string|bool|null)
does not emit InvalidCast — the scalar atoms make the cast valid
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function option(string $key): string|array|bool|null {
    return null;
}

$timeout = (int) option('timeout');
$retries = (int) option('retries');
===expect===
