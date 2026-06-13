===description===
ForbiddenCode fires for backtick shell_exec.
===config===
suppress=UnusedParam
===file===
<?php
function run(string $cmd): string {
    return `$cmd`;
}
===expect===
ForbiddenCode@3:12-3:18: Use of shell_exec (backtick) is forbidden
