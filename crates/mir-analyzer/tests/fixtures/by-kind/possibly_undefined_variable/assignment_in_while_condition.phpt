===description===
An assignment in the `&&`-guarded `while` condition defines `$line` in the body.
===config===
<mir>
  <issueHandlers>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
function run(mixed $resource): void {
    while (!feof($resource) && ($line = fgets($resource))) {
        echo strlen($line);
    }
}
===expect===
