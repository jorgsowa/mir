===description===
Writing a literal key on a shape-typed property makes that key definitely
set for later reads of the same property, including optional keys and
nested receivers.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    /** @var array{id: int, label?: string} */
    private array $data = ['id' => 1];

    public function viaThis(): string {
        $this->data['label'] = 'x';
        return $this->data['label'];
    }

    public function viaParam(Box $other): string {
        $other->data['label'] = 'x';
        return $other->data['label'];
    }

    public function overwriteRequired(): int {
        $this->data['id'] = 2;
        return $this->data['id'];
    }
}
