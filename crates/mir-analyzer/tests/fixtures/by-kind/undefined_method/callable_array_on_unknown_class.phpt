===description===
A callable array reports a missing method on the receiver's own class, and nothing when that class is unknown.
===config===
<mir>
  <issueHandlers>
    <UndefinedDocblockClass errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App\Sub;

class P {
    /** @param \Missing\Vendor\Lexer $lexer */
    public function f($lexer, array $t): array {
        $lexer->getLiteral(1);
        return array_map([$lexer, 'getLiteral'], $t);
    }
    public function g(\DateTime $d, array $t): array {
        return array_map([$d, 'nope'], $t);
//                       ^^^^^^^^^^^^ UndefinedMethod: Method DateTime::nope() does not exist
    }
}
