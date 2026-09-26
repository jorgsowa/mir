===description===
References to a promoted property include `$this->` and external fetches.
===cursor===
references
===file===
<?php
final class User {
    public function __construct(public string $name) {}
    public function upper(): string { return strtoupper($this->name); }
}
echo (new User('x'))->na<CURSOR>me;
===expect===
test.php@4:63-4:67
test.php@6:22-6:26
