===description===
A property fetch lands on the property even when a method shares its name.
===ignore===
===cursor===
definition
===file===
<?php
final class User {
    private string $name = 'x';
    public function name(): string { return $this->na<CURSOR>me; }
}
===expect===
test.php@3:4-3:30
