===description===
Forbidding only var_dump leaves shell_exec and the backtick operator alone.
===config===
<mir>
  <forbiddenFunctions>
    <function name="var_dump"/>
  </forbiddenFunctions>
</mir>
===file===
<?php
function run(string $cmd): void {
    shell_exec($cmd);
    $_ = `ls`;
    var_dump($cmd);
//  ^^^^^^^^^^^^^^ ForbiddenCode: Use of var_dump is forbidden
}
