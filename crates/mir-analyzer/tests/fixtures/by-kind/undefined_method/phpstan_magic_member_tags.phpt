===description===
`@phpstan-method` and `@phpstan-property*` declare magic members, and a prefixed tag overrides the plain one for the same name.
===file===
<?php
/**
 * @phpstan-method int count()
 * @phpstan-property int $id
 * @phpstan-property-read string $label
 * @phpstan-property-write string $note
 * @method string size()
 * @phpstan-method int size()
 * @property string $total
 * @phpstan-property int $total
 */
class Model {}

function count_of(Model $m): int {
    return $m->count();
}

function id_of(Model $m): int {
    return $m->id;
}

function label_of(Model $m): string {
    return $m->label;
}

function set_note(Model $m): void {
    $m->note = 'x';
}

function size_of(Model $m): string {
    return $m->size();
//  ^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
}

function total_of(Model $m): string {
    return $m->total;
//  ^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
}
