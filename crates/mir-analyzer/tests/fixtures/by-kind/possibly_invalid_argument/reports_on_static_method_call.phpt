===description===
reports on static method call
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Converter {
    public static function process(string $s): void { var_dump($s); }
}
/** @return string|false */
function readInput(): string|false { return 'data'; }
function test(): void {
    Converter::process(readInput());
//                     ^^^^^^^^^^^ PossiblyInvalidArgument: Argument $s of process() expects 'string', possibly different type 'string|false' provided
}
===expect===
