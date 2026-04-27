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
PossiblyInvalidArgument: Argument $input of parse() expects 'string', possibly different type 'string|false' provided
