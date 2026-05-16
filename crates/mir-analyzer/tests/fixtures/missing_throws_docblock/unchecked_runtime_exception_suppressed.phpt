===description===
RuntimeException and descendants are unchecked — no @throws warning by default
===file===
<?php
function fa(): void {
    throw new \RuntimeException('fail');
}

function fb(): void {
    throw new \InvalidArgumentException('bad arg');
}

function fc(): void {
    throw new \LogicException('logic');
}

function fd(): void {
    throw new \DomainException('domain');
}

class MyCustomError extends \RuntimeException {}

function fe(): void {
    throw new MyCustomError('custom');
}
===expect===
