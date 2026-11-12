===description===
`+` between two operands that may both be arrays is valid for their array members: a nullable array is PossiblyNullOperand, other non-array members are PossiblyInvalidOperand, and an operand that is never an array stays InvalidOperand.
===file===
<?php
interface Definition {}

/** @param array|Definition $definition */
function partial($definition): array {
    return ['id' => 1] + $definition;
//         ^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidOperand: Operator '+' might not be supported between 'array{'id': 1}' and 'array|Definition'
}

function nullable(?array $options): array {
    return ['all' => 'Any'] + $options;
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullOperand: Operator '+' operand 'array|null' might be null
}

function parsed(string $url): mixed {
    return parse_url($url) + ['query' => false];
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidOperand: Operator '+' might not be supported between 'array|string|int|null|false' and 'array{'query': false}'
}

function never(Definition $definition): array {
    return ['id' => 1] + $definition;
//         ^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOperand: Operator '+' not supported between 'array{'id': 1}' and 'Definition'
}
