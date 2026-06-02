===description===
Enum string or enum int without spaces incorrect
===file===
<?php
namespace Ns;

/** @param "foo"with"|"bar"|1|2|3 $s */
function foo($s) : void {}
foo(4);
===expect===
InvalidArgument@6:5-6:6: Argument $s of foo() expects '"foo"with"|"bar"|1|2|3', got '4'
