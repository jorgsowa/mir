===description===
An object with __toString() loosely compares to a string by its string value.
===file===
<?php
final class Alias {
    public function __toString(): string { return 'a'; }
}
/** @param array<string, Alias> $aliases */
function find(string $id, array $aliases): ?string {
    foreach ($aliases as $alias => $origin) {
        if ($id == $origin) {
            return $alias;
        }
    }
    return null;
}
