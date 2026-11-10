===description===
`finally` runs after a `return` nested inside the `try` body, so it sees the variables as they were at that return; a value no path assigns is still impossible.
===file===
<?php
function propertyType(object $subject, string $property): ?string {
    $type = null;
    try {
        if (($reflection = new \ReflectionProperty($subject, $property))->isPublic()) {
            $type = $reflection->getType();
            return 'public';
        }
        return null;
    } catch (\ReflectionException) {
        return null;
    } finally {
        echo $type instanceof \ReflectionNamedType ? $type->getName() : 'untyped';
    }
}

function tracked(bool $ok): string {
    $state = 'idle';
    try {
        if ($ok) {
            $state = 'done';
            return 'ok';
        }
        return 'no';
    } finally {
        if ($state === 'done') {
            echo 'finished';
        }
    }
}

function neverAssigned(bool $ok): string {
    $state = 'idle';
    try {
        if ($ok) {
            return 'ok';
        }
        return 'no';
    } finally {
        echo $state === 'done' ? 'finished' : 'pending';
//           ^^^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"idle"' and '"done"' is always false — these types can never be identical
    }
}

function returnInCatch(string $json): string {
    $error = null;
    try {
        return (string) json_encode(json_decode($json, flags: \JSON_THROW_ON_ERROR));
    } catch (\JsonException $e) {
        $error = $e->getMessage();
        return '';
    } finally {
        echo $error === null ? 'valid' : $error;
    }
}
