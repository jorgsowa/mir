===description===
template-conditional return (TKey is null ? X : Y) resolves to if-false when non-null is passed
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
$result = $repo->fetchRows("id");
/** @mir-check $result is array<int, array<string, mixed>> */
echo "ok";
===expect===
