===description===
A cursor at the very end of the file resolves nothing.
===cursor===
symbol
===file===
<?php
echo 1;
<CURSOR>
===expect===
error: NotFound
