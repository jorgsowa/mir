===description===
ForbiddenCode fires for backtick shell_exec.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run(string $cmd): string {
    return `$cmd`;
//         ^^^^^^ ForbiddenCode: Use of shell_exec (backtick) is forbidden
}
===expect===
