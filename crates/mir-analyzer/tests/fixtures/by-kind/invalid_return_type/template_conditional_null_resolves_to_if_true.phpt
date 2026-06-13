===description===
template-conditional return (TKey is null ? X : Y) resolves to if-true when null is passed
===config===
suppress=UnusedParam,UnusedVariable
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

$repo = new Repo();
$result = $repo->fetchRows(null);
/** @mir-check $result is list<array<string, mixed>> */
echo "ok";
===expect===
