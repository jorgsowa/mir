===description===
MismatchingDocblockReturnType fires when the @return docblock declares array
but the native hint is string (incompatible type families).
===file===
<?php
/** @return array */
function arrayDocStringHint(): string { return 'x'; }
===expect===
MismatchingDocblockReturnType@3:9-3:27: Docblock return type 'array<mixed, mixed>' does not match inferred 'string'
InvalidReturnType@3:40-3:51: Return type '"x"' is not compatible with declared 'array<mixed, mixed>'
