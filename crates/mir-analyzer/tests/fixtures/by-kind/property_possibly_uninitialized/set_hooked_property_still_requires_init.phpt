===description===
Set-hooked properties still require initialization.
===config===
php_version=8.4
===file===
<?php
final class Box {
    public string $value {
        get => $this->value;
        set => $this->value = strtoupper($value);
    }

    public function __construct() {
    }
}
===expect===
PropertyPossiblyUninitialized@8:20-8:31: Property Box::$value may be left uninitialized by the constructor
