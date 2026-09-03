===description===
FALSE POSITIVE reproducer. Valid PHP: Property is non-null after the `=== null` branch assigns it.
mir 0.42.0 currently emits (the bug): NullableReturnStatement@9:8-9:29: expected string, actual string|null
Expected: no issue. Remove ===ignore=== to activate once fixed.
===config===
php_version=8.4
===file===
<?php
class Cache {
    private ?string $client = null;
    public function client(): string {
        // FP expected: NullableReturnStatement (assignment makes it non-null)
        if ($this->client === null) {
            $this->client = 'built';
        }
        return $this->client;
    }
}
===expect===
