===description===
With no forbiddenFunctions config, var_dump, shell_exec and the backtick operator are not reported.
===file===
<?php
function run(string $cmd): void {
    var_dump($cmd);
    shell_exec($cmd);
    $_ = `ls`;
}
===expect===
