===description===
A shape whose keys are all optional may be empty, so its truthiness is undecided; a shape with a required key is always truthy.
===file===
<?php
function query(?string $from, ?string $source): string {
    $query = array_filter(['from' => $from, 'source' => $source]);
    return $query ? '?' . http_build_query($query) : '';
}

/** @param array{page?: int, limit?: int} $options */
function optionalKeys(array $options): string {
    if ($options) {
        return 'custom';
    }
    return 'default';
}

/** @param array{page: int, limit?: int} $options */
function requiredKey(array $options): string {
    return $options ? 'custom' : 'default';
//         ^^^^^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
}
