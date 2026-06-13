===description===
Class constant incorrect
===config===
suppress=UnusedParam
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
InvalidArgument@10:5-10:10: Argument $s of foo() expects '"foo"|"bar"|Ns\C::A|Ns\C::B', got '"for"'
