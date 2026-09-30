===description===
MismatchingDocblockReturnType fires when the @return docblock declares array
but the native hint is string (incompatible type families).
===file===
<?php
/** @return array */
function arrayDocStringHint(): string { return 'x'; }
//       ^^^^^^^^^^^^^^^^^^ MismatchingDocblockReturnType: Docblock return type 'array' does not match inferred 'string'
//                                      ^^^^^^^^^^^ InvalidReturnType: Return type '"x"' is not compatible with declared 'array'
===expect===
