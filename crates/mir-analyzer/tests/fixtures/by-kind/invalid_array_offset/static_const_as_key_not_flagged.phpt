===description===
static::INT_CONST used as array key does not emit InvalidArrayOffset.
static:: resolves to the current class and returns the literal type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Registry {
    const KEY = 42;

    /** @var array<int, bool> */
    private array $data = [];

    public function mark(): void {
        $this->data[static::KEY] = true;
    }
}
===expect===
