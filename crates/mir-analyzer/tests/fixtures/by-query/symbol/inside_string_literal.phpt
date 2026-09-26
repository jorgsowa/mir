===description===
A cursor inside a string literal resolves nothing.
===cursor===
symbol
===file===
<?php
echo strlen('ab<CURSOR>c');
===expect===
error: NotFound
