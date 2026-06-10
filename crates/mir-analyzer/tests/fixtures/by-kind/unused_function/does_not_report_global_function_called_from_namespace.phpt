===description===
does not report global function called from namespace
===file===
<?php
function helper(): void {}

namespace App;

\helper();
===expect===
ParseError@2:1-2:27: Parse error: Namespace declaration statement has to be the very first statement or after any declare call in the script
