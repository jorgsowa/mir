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
PossiblyInvalidArgument: Argument $s of process() expects 'string', possibly different type 'string|false' provided
