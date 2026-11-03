===description===
Writes count when they reserve memory (`str_repeat`) or when native serialization reads the property
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class ErrorHandler {
    private ?string $reservedMemory = null;
    private static ?string $staticReserve = null;

    public function register(): void {
        $this->reservedMemory = str_repeat(' ', 20 * 1024);
        self::$staticReserve = \str_repeat('x', 32768);
    }
}

final class Location {
    private int $line = 0;

    public function __construct(int $line) {
        $this->line = $line;
    }

    public function __unserialize(array $data): void {
        $this->line = (int) $data['line'];
    }
}

(new ErrorHandler())->register();
serialize(new Location(1));
