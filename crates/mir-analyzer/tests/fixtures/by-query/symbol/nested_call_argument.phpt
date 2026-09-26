===description===
A call nested in another call's arguments resolves to the inner call.
===cursor===
symbol
===file===
<?php
echo strlen(tr<CURSOR>im(' a '));
===expect===
kind: function call trim
type: string
