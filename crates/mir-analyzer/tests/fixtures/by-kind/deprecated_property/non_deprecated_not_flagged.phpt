===description===
DeprecatedProperty does NOT fire for non-deprecated properties.
===file===
<?php
class Config {
    public string $host = "localhost";
}

$c = new Config();
echo $c->host;
===expect===
