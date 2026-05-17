===description===
Assigning to a global variable is an externally observable side effect and must not be reported as UnusedVariable.
===file===
<?php
function loadConfig(): array { return []; }
function setup(): void {
    global $config;
    $config = loadConfig();
}
===expect===
