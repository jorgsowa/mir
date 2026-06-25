===description===
Under strict_types=1, passing an object with __toString to a string param is InvalidArgument — PHP does not coerce
===config===
suppress=UnusedParam
===file===
<?php declare(strict_types=1);
class Label {
    public function __toString(): string { return 'Label'; }
}

function render(string $s): void {}

render(new Label());
===expect===
InvalidArgument@8:7-8:18: Argument $s of render() expects 'string', got 'Label'
