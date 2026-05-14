===description===
static constant access via bare FQN without use statement produces no error
===file:Config.php===
<?php
class Config {
    public const DEBUG = false;
    public const VERSION = '1.0';
}
===file:App.php===
<?php
function boot(): void {
    if (\Config::DEBUG) {
        echo \Config::VERSION;
    }
}
===expect===
