===description===
FN: trait properties never checked the #[Deprecated] attribute fallback,
unlike class properties — only the @deprecated docblock tag worked.
===config===
suppress=MissingPropertyType
===file===
<?php
trait HasServer {
    #[Deprecated]
    public $server = "localhost";
}

class Config {
    use HasServer;
}

$c = new Config();
echo $c->server;
===expect===
DeprecatedProperty@12:9-12:15: Property Config::$server is deprecated
