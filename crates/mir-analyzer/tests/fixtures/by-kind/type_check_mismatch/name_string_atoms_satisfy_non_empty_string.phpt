===description===
M16: class-string, interface-string, callable-string, enum-string, and
trait-string are all always non-empty in real PHP (a class/interface/
callable/enum/trait name can never be "") — each must satisfy a
non-empty-string param.
===config===
suppress=UnusedParam,UndefinedClass
===file===
<?php
interface Greeter {}
enum Status { case Active; }
trait HasName {}

/** @param non-empty-string $s */
function requireNonEmpty(string $s): void {}

/** @param class-string $c */
function classString(string $c): void { requireNonEmpty($c); }

/** @param interface-string $i */
function interfaceString(string $i): void { requireNonEmpty($i); }

/** @param callable-string $f */
function callableString(string $f): void { requireNonEmpty($f); }

/** @param enum-string $e */
function enumString(string $e): void { requireNonEmpty($e); }

/** @param trait-string $t */
function traitString(string $t): void { requireNonEmpty($t); }
===expect===
