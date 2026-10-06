===description===
`array_keys(get_defined_constants())` returns string keys.
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function dump(): void
{
    foreach (array_keys(get_defined_constants()) as $key) {
        takesString($key);
    }
}

function takesString(string $s): void
{
    var_dump($s);
}
