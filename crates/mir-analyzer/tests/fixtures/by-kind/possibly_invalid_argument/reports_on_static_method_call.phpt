===description===
reports on static method call
===config===
suppress=ForbiddenCode
===file===
<?php
class Converter {
    public static function process(string $s): void { var_dump($s); }
}
/** @return string|false */
function readInput(): string|false { return 'data'; }
function test(): void {
    Converter::process(readInput());
}
===expect===
PossiblyInvalidArgument@8:23-8:34: Argument $s of process() expects 'string', possibly different type 'string|false' provided
