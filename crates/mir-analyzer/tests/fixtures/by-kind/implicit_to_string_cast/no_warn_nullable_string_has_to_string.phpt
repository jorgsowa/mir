===description===
No warning when __toString object is passed to a nullable string (?string) parameter
===config===
suppress=UnusedParam
===file===
<?php
class Name {
    public function __toString(): string { return 'name'; }
}

function greet(?string $value): void {}

greet(new Name());
===expect===
