===description===
A value typed interface-string<T> satisfies a plain class-string parameter:
every interface-string is a valid class-string at runtime.
===config===
suppress=MissingReturnType,UnusedVariable
===file===
<?php
interface Shape {}

/** @param class-string $className */
function needsClassString(string $className) {
    return $className;
}

function forward(string $iface): void {
    /** @var interface-string<Shape> $iface */
    needsClassString($iface);
}
===expect===
