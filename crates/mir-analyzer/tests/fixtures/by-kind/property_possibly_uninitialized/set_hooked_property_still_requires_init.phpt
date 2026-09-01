===description===
A property with a `set` hook (with or without `get`) is backed by real
storage — the hook only intercepts assignment, it doesn't remove the need
for one. Unlike a get-only hook, it can still be left uninitialized.
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
