===description===
Class constant incorrect
===file===
<?php
namespace Ns;

class C {
    const A = "bat";
    const B = "baz";
}
/** @param "foo"|"bar"|C::A|C::B $s */
function foo($s) : void {}
foo("for");
===expect===
InvalidArgument
===ignore===
TODO
