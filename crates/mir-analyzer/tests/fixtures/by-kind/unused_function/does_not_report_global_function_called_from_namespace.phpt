===description===
does not report global function called from namespace
===file===
<?php
function helper(): void {}

namespace App;

\helper();
===expect===
===ignore===
TODO
