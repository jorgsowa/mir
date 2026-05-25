===description===
Enum string or enum int without spaces incorrect
===file===
<?php
namespace Ns;

/** @param "foo"with"|"bar"|1|2|3 $s */
function foo($s) : void {}
foo(4);
===expect===
InvalidArgument
===ignore===
TODO
