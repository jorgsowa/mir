===description===
No ImplicitToStringCast when the enum implements Stringable. PHP rejects `__toString` on an enum, so the declaration is a compile error.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Suit implements Stringable {
    case Hearts;

    public function __toString(): string {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Enum Suit cannot include magic method __toString [enum-magic-method]
        return 'Hearts';
    }
}
$s = 'Suit: ' . Suit::Hearts;
echo Suit::Hearts;
