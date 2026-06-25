===description===
DeprecatedProperty fires with no message suffix when @deprecated tag has no text.
===file===
<?php
class Config {
    /** @deprecated */
    public string $server = "localhost";
}

$c = new Config();
echo $c->server;
===expect===
DeprecatedProperty@8:9-8:15: Property Config::$server is deprecated
