===description===
A native-typed, non-nullable, default-less property never assigned anywhere
in the constructor throws PHP's "must not be accessed before initialization"
on first read — flag it at the constructor itself.
===file===
<?php
class Config {
    public string $env;
    public string $version;

    public function __construct(string $env) {
//                  ^^^^^^^^^^^ PropertyPossiblyUninitialized: Property Config::$version may be left uninitialized by the constructor
        $this->env = $env;
    }
}
===expect===
