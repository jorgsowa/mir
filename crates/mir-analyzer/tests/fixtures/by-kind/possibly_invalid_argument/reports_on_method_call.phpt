===description===
reports on method call
===config===
suppress=ForbiddenCode
===file===
<?php
class Parser {
    public function parse(string $input): void { var_dump($input); }
}
/** @return string|false */
function readInput(): string|false { return 'data'; }
function test(Parser $parser): void {
    $parser->parse(readInput());
}
===expect===
PossiblyInvalidArgument@8:19-8:30: Argument $input of parse() expects 'string', possibly different type 'string|false' provided
