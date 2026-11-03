===description===
Constants used by other declarations' initializers, defaults, enum cases and attributes are used
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
#[Attribute]
final class Tag {
    public function __construct(public int $v) {}
}

final class Holder {
    private const BASE = 10;
    private const FOR_PROP = 1;
    private const FOR_PARAM = 2;
    private const FOR_STATIC = 3;
    private const FOR_ATTR = 4;
    private const FOR_MATCH = 5;
    public const DERIVED = self::BASE * 2;

    private int $p = self::FOR_PROP;

    #[Tag(self::FOR_ATTR)]
    public function run(int $x = self::FOR_PARAM): int {
        static $s = self::FOR_STATIC;
        return match ($x) {
            self::FOR_MATCH => $s,
            default => $this->p,
        };
    }
}

enum Level: int {
    private const LOW = 1;
    case Low = self::LOW;
}

(new Holder())->run();
echo Level::Low->value;
