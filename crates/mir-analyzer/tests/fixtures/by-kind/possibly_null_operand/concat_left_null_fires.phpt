===description===
PossiblyNullOperand fires when the left operand of concatenation might be null.
===file===
<?php
function combine(?string $pfx, string $x): string {
    return $pfx . $x;
}
===expect===
PossiblyNullOperand@3:11-3:15: Operator '.' operand 'string|null' might be null
