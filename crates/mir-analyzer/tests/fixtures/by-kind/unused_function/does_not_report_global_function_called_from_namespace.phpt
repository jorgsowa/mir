===description===
does not report global function called from namespace
===file===
<?php
function helper(): void {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ ParseError: Parse error: Namespace declaration statement has to be the very first statement or after any declare call in the script

namespace App;

\helper();
===expect===
