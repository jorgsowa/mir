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
//       ^^^^^^ DeprecatedProperty: Property Config::$server is deprecated
===expect===
