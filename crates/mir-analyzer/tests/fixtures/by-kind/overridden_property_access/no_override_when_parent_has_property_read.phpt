===description===
@property-read and @property-write docblock tags are also virtual and must not
trigger OverriddenPropertyAccess when a child class declares a real property
with the same name.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

/**
 * @property-read int $id
 * @property-write string $label
 */
class Base {
    public function __get(string $key): mixed { return null; }
    public function __set(string $key, mixed $value): void {}
}

class Derived extends Base {
    private int $id = 0;
    protected string $label = '';
}
