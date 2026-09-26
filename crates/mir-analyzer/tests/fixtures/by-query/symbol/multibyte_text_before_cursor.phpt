===description===
Byte offsets after multi-byte UTF-8 text still land on the right token.
===cursor===
symbol
===file===
<?php
echo 'żółć'; echo str<CURSOR>len('x');
===expect===
kind: function call strlen
type: 1
