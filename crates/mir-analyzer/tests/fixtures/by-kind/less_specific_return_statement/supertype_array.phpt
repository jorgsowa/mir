===description===
Returning a generic array whose values are wider than declared is LessSpecificReturnStatement; an empty array for a non-empty return type or a non-array stays InvalidReturnType.
===file===
<?php
interface Entity {}
interface Role extends Entity {}

/**
 * @param list<Entity> $entities
 * @return list<Role>
 */
function roles(array $entities): array {
    return $entities;
//  ^^^^^^^^^^^^^^^^^ LessSpecificReturnStatement: Return type 'list<Entity>' is less specific than declared 'list<Role>'
}

/**
 * @param array<string, Entity> $byName
 * @return array<string, Role>
 */
function byName(array $byName): array {
    return $byName;
//  ^^^^^^^^^^^^^^^ LessSpecificReturnStatement: Return type 'array<string, Entity>' is less specific than declared 'array<string, Role>'
}

/** @return non-empty-list<int> */
function nonEmpty(): array {
    return [];
//  ^^^^^^^^^^ InvalidReturnType: Return type 'array{}' is not compatible with declared 'non-empty-list<int>'
}

/** @return list<int> */
function scalar(): array {
    return 'x';
//  ^^^^^^^^^^^ InvalidReturnType: Return type '"x"' is not compatible with declared 'list<int>'
}
