===description===
An unqualified call in a namespace falls back to the global function.
===cursor===
symbol
===file===
<?php
namespace App;

echo str<CURSOR>len('abc');
===expect===
kind: function call strlen
type: 3
