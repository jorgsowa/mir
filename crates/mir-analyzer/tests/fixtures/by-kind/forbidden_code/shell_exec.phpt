===description===
ForbiddenCode fires for backtick shell_exec when shell_exec is configured as forbidden.
===config===
<mir>
  <forbiddenFunctions>
    <function name="shell_exec"/>
  </forbiddenFunctions>
</mir>
===file===
<?php
function run(string $cmd): string {
    return `$cmd`;
//         ^^^^^^ ForbiddenCode: Use of shell_exec (backtick) is forbidden
}
===expect===
