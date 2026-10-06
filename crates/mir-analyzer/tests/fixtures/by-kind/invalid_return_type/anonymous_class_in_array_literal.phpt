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
//  ^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{0: object}' is not compatible with declared 'list<int>'
}

/** @return list<string> */
function notAStringElement(): array {
    $h = new class {};
    return [$h];
//  ^^^^^^^^^^^^ InvalidReturnType: Return type 'array{0: object}' is not compatible with declared 'list<string>'
}
