===description===
The `array_key_exists('favicon', ...)` guard proves the offset exists before access.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
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
