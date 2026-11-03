===description===
Writing a new key into a property narrowed to a shape adds the key to the narrowed shape.
===file===
<?php
final class Firewall {
    /** @return list<string> */
    public function listeners(): array { return []; }
}
final class Collector {
    /** @var array<string, mixed> */
    protected array $data = [];

    public function __construct(private ?Firewall $firewall) {}

    public function collect(): bool {
        $this->data = ['enabled' => true];
        $this->data['firewall'] = null;
        $this->data['listeners'] = [];
        if ($this->firewall) {
            $this->data['listeners'] = $this->firewall->listeners();
        }
        return $this->data['listeners'] !== [] && $this->data['firewall'] === null;
    }
}
