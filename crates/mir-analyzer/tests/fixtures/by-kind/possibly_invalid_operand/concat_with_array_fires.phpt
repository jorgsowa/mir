===description===
PossiblyInvalidOperand fires for concatenation when a union member is an array.
===file===
<?php
function build(string $pfx, string|array $parts): string {
    return $pfx . $parts;
}
===expect===
PossiblyInvalidOperand@3:11-3:24: Operator '.' might not be supported between 'string' and 'string|array<mixed, mixed>'
