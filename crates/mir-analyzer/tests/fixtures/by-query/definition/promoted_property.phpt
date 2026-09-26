===description===
A constructor-promoted property lands on the promoted parameter.
===cursor===
definition
===file===
<?php
final class User {
    public function __construct(public string $name) {}
}
echo (new User('x'))->na<CURSOR>me;
===expect===
test.php@3:32-3:51
