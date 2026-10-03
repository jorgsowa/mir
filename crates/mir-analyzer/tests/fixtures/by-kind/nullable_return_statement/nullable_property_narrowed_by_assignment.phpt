===description===
Property is non-null after the `=== null` branch assigns it.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
class Cache {
    private ?string $client = null;
    public function client(): string {
        if ($this->client === null) {
            $this->client = 'built';
        }
        return $this->client;
    }
}
===expect===
