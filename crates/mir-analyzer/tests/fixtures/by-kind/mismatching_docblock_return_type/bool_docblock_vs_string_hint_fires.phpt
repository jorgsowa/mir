===description===
MismatchingDocblockReturnType fires when the @return docblock declares bool
but the native hint is string (incompatible type families).
===file===
<?php
/** @return bool */
function boolDocStringHint(): string { return 'x'; }
===expect===
MismatchingDocblockReturnType@3:9-3:26: Docblock return type 'bool' does not match inferred 'string'
InvalidReturnType@3:39-3:50: Return type '"x"' is not compatible with declared 'bool'
