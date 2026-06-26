===description===
PossiblyNullOperand fires when the right operand of concatenation might be null.
===file===
<?php
function combine(string $x, ?string $sfx): string {
    return $x . $sfx;
}
===expect===
PossiblyNullOperand@3:16-3:20: Operator '.' operand 'string|null' might be null
