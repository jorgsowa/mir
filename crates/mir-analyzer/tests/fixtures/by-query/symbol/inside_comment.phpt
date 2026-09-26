===description===
A cursor inside a comment resolves nothing.
===cursor===
symbol
===file===
<?php
// calls str<CURSOR>len
echo strlen('abc');
===expect===
error: NotFound
