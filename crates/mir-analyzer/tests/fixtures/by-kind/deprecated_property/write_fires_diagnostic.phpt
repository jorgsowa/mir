===description===
DeprecatedProperty fires when assigning (writing) to a deprecated property, not only on read.
===file===
<?php
class Config {
    /**
     * @deprecated Use $host instead.
     */
    public string $server = "localhost";
}

$c = new Config();
$c->server = "new-host";
===expect===
DeprecatedProperty@10:0-10:23: Property Config::$server is deprecated: Use $host instead.
