===description===
ForbiddenCode fires for backtick shell_exec.
===config===
suppress=UnusedParam
===file===
<?php
function run(string $cmd): string {
    return `$cmd`;
//         ^^^^^^ ForbiddenCode: Use of shell_exec (backtick) is forbidden
}
===expect===
