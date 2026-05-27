===description===
template-conditional return with nullable discriminator widens to if_true|if_false at the call site — no InvalidArgument when the union is passed to a function expecting array
===config===
suppress=UnusedParam,UnusedVariable,UnusedFunction
===file===
<?php

class Repo {
    /**
     * @template TKey of non-empty-string|null
     * @param TKey $keyColumn
     * @return (TKey is null ? list<array<string, mixed>> : array<int, array<string, mixed>>)
     */
    public function fetchRows(mixed $keyColumn): array {
        return [];
    }
}

/** @param non-empty-string|null $key */
function consume(string|null $key): void {
    $repo = new Repo();
    $rows = $repo->fetchRows($key);
    // Both branches are subtypes of array — must not produce InvalidArgument.
    array_map(fn($row) => $row, $rows);
}
===expect===
