===description===
FALSE POSITIVE reproducer. Valid PHP: The `array_key_exists('favicon', ...)` guard proves the offset exists before access.
mir 0.42.0 currently emits (the bug): NonExistentArrayOffset@7:79-7:88: 'favicon'
Expected: no issue. Remove ===ignore=== to activate once fixed.
===config===
php_version=8.4
===file===
<?php
class Renderer {
    /** @var array{title: string} */
    private array $meta = ['title' => 't'];
    public function favicon(): string {
        // expect: NonExistentArrayOffset 'favicon' despite array_key_exists guard
        return array_key_exists('favicon', $this->meta) ? (string) $this->meta['favicon'] : '';
    }
}
===expect===
