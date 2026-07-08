===description===
MismatchingDocblockReturnType fires when the @return docblock declares bool
but the native hint is string (incompatible type families). The function's
own `return 'x';` must NOT also trigger InvalidReturnType — the native hint
is runtime truth regardless of what the docblock (wrongly) claims.
===file===
<?php
/** @return bool */
function boolDocStringHint(): string { return 'x'; }
===expect===
MismatchingDocblockReturnType@3:9-3:26: Docblock return type 'bool' does not match inferred 'string'
