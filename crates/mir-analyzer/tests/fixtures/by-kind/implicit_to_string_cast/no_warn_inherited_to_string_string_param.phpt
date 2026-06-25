===description===
No warning when object inherits __toString from a parent and is passed to a string param
===config===
suppress=UnusedParam
===file===
<?php
class Base {
    public function __toString(): string { return 'base'; }
}
class Child extends Base {}

function render(string $s): void {}

render(new Child());
===expect===
