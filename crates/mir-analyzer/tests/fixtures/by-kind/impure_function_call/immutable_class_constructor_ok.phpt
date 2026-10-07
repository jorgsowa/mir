===description===
Constructing an immutable class from an immutable method is allowed even when
a mutable object is passed in: its constructor only initializes the new
instance. A constructor inherited from a mutable parent stays flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedClass errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Registry {
    /** @var list<string> */
    public array $items = [];
}

class MutableBase {
    public function __construct(public Registry $registry) {}
}

/** @psalm-immutable */
class Settings {
    public function __construct(private int $limit, private Registry $registry) {}

    public function withLimit(int $limit): self {
        $copy = new self($limit, $this->registry);
        /** @mir-check $copy is Settings */
        return $copy;
    }

    public function strict(): StrictSettings {
        return new StrictSettings($this->limit, $this->registry);
    }

    public function legacy(): LegacySettings {
        return new LegacySettings($this->registry);
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function LegacySettings::__construct() in a @pure function
    }
}

/** @psalm-immutable */
final class StrictSettings extends Settings {}

/** @psalm-immutable */
final class LegacySettings extends MutableBase {}
