===description===
does not report global function called from namespace
===config===
find_dead_code=true
===file===
<?php
function helper(): void {}

namespace App;

\helper();
===expect===
===ignore===
TODO
