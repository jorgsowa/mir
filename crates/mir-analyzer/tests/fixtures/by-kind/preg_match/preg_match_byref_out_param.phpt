===description===
`preg_match($re,$s,$m)` defines `$m` by reference when it returns 1.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
function run(string $s): void {
    if (str_contains($s, 'x') && preg_match('/(\d+)/', $s, $m) === 1) {
        echo $m[1];
    }
}
