===description===
Constant initializers referencing their own class-like (`self::`, its own name, `parent::`) are typed, in any declaration order
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;
class P { const BASE = 10; }
class C extends P {
    const LATER = self::EARLY + 1;
    const EARLY = 1;
    const BY_NAME = \App\C::EARLY;
    const FROM_PARENT = parent::BASE;
    const CYC1 = self::CYC2;
    const CYC2 = self::CYC1;
}
enum Suit { case H; const DEFAULT = self::H; }
interface I { const IA = 'i'; const IB = self::IA . 'x'; }
trait T { const TA = 5; const TB = self::TA; }
class U { use T; }
function takes(array $_a): void {}
function t(): void {
    takes(C::LATER);
//        ^^^^^^^^ InvalidArgument: Argument $_a of takes() expects 'array', got '2'
    takes(C::BY_NAME);
//        ^^^^^^^^^^ InvalidArgument: Argument $_a of takes() expects 'array', got '1'
    takes(C::FROM_PARENT);
//        ^^^^^^^^^^^^^^ InvalidArgument: Argument $_a of takes() expects 'array', got '10'
    takes(C::CYC1);
//        ^^^^^^^ MixedArgument: Argument $_a of takes() is mixed
    takes(Suit::DEFAULT);
//        ^^^^^^^^^^^^^ InvalidArgument: Argument $_a of takes() expects 'array', got 'App\Suit'
    takes(I::IB);
//        ^^^^^ InvalidArgument: Argument $_a of takes() expects 'array', got '"ix"'
    takes(U::TB);
//        ^^^^^ InvalidArgument: Argument $_a of takes() expects 'array', got '5'
}
