===description===
DeprecatedProperty fires when accessing a @deprecated property.
===file===
<?php
class Config {
    /**
     * @deprecated Use $host instead.
     */
    public string $server = "localhost";
    public string $host = "localhost";
}

$c = new Config();
echo $c->server;
===expect===
DeprecatedProperty@11:10-11:16: Property Config::$server is deprecated: Use $host instead.
