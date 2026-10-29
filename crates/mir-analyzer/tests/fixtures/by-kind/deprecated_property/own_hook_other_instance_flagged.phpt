===description===
Only `$this->prop` is the backing value inside the property's own hooks; the same property on another instance runs that instance's hooks and is still a use.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
final class Event {
    public ?Event $prev = null;

    /** @deprecated use $id */
    public ?int $legacy {
        get => $this->prev?->legacy ?? $this->legacy;
//                           ^^^^^^ DeprecatedProperty: Property Event::$legacy is deprecated: use $id
        set {
            if ($this->prev !== null) {
                $this->prev->legacy = $value;
//              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedProperty: Property Event::$legacy is deprecated: use $id
            }
            $this->legacy = $value;
        }
    }
}
