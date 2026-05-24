===description===
classConstantInvalidValue
===file===
<?php
namespace NS {
    use OtherNSC as E;
    class C {}
    class D {};
    class F {};
    /** @psalm-param C::class|D::class|E::class $s */
    function foo(string $s) : void {}
    foo(F::class);
}

namespace OtherNS {
    class C {}
}
===expect===
InvalidArgument
===ignore===
TODO
