===description===
Inside a deprecated property's own hooks `$this->prop` is the backing value, not a use of the property; other hooks, methods and callers still get `DeprecatedProperty`.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
final class Event {
    /** @deprecated use $id */
    public private(set) ?int $legacy {
        get {
            if ($this->id === 0) {
                return null;
            }
            return $this->legacy ??= $this->id;
        }
        set { $this->legacy = $value; }
    }

    /** @deprecated */
    public ?int $old = null;

    public ?int $alias { get => $this->old; }
//                                     ^^^ DeprecatedProperty: Property Event::$old is deprecated

    public function __construct(public int $id) {}

    public function read(): ?int { return $this->legacy; }
//                                               ^^^^^^ DeprecatedProperty: Property Event::$legacy is deprecated: use $id
}

echo (new Event(1))->legacy;
//                   ^^^^^^ DeprecatedProperty: Property Event::$legacy is deprecated: use $id
