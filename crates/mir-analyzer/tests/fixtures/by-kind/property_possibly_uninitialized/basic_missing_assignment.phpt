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
        $this->env = $env;
    }
}
===expect===
PropertyPossiblyUninitialized@6:20-6:31: Property Config::$version may be left uninitialized by the constructor
