===description===
An anonymous class inside an array literal fits a named-class element type
===file===
<?php
interface Handler {}

/** @return list<Handler> */
function inList(): array {
    return [new class implements Handler {}];
}

/** @return list<\Throwable> */
function extendsBuiltin(): array {
    return [new class('x') extends \DomainException {}];
}

/** @return array<string, Handler> */
function inMap(): array {
    return ['k' => new class implements Handler {}];
}

/** @return array<string, list<Handler>> */
function nested(): array {
    return ['k' => [new class implements Handler {}]];
}

/** @return list<Handler> */
function viaVariable(): array {
    $h = new class implements Handler {};
    return [$h];
}

/** @return list<int> */
function notAnObjectElement(): array {
    return [new class {}];
}

/** @return list<string> */
function notAStringElement(): array {
    $h = new class {};
    return [$h];
}
===expect===
InvalidReturnType@32:4-32:26: Return type 'array{0: object}' is not compatible with declared 'list<int>'
InvalidReturnType@38:4-38:16: Return type 'array{0: object}' is not compatible with declared 'list<string>'
